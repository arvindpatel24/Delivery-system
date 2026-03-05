use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use delivery_domain::entities::batch::{BatchCluster, BatchRun};
use delivery_domain::entities::order::{OrderStatus, RoutingMode};
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::{
    BatchRepository, DriverRepository, GeospatialEngine, OrderRepository,
};

pub struct BatchService {
    pub order_repo: Arc<dyn OrderRepository>,
    pub batch_repo: Arc<dyn BatchRepository>,
    pub driver_repo: Arc<dyn DriverRepository>,
    pub geospatial: Arc<dyn GeospatialEngine>,
    pub cluster_eps_meters: f64,
    pub max_orders_per_cluster: usize,
}

impl BatchService {
    pub async fn run_batch(&self, scheduled_hour: i32) -> DomainResult<Uuid> {
        let pending = self
            .order_repo
            .find_pending_by_routing_mode(RoutingMode::Batched)
            .await?;

        if pending.is_empty() {
            tracing::info!("No pending batched orders to process");
            return Err(DomainError::Validation(
                "No pending batched orders".to_string(),
            ));
        }

        let now = Utc::now();
        let run = BatchRun {
            id: Uuid::now_v7(),
            scheduled_hour,
            total_orders: pending.len() as i32,
            total_clusters: 0,
            total_drivers_assigned: 0,
            started_at: now,
            completed_at: None,
        };
        let run = self.batch_repo.create_run(&run).await?;

        let order_ids: Vec<Uuid> = pending.iter().map(|o| o.id).collect();
        let clusters = self
            .geospatial
            .cluster_orders(&order_ids, self.cluster_eps_meters, 1)
            .await?;

        let mut total_drivers = 0i32;
        let mut total_clusters = 0i32;

        for cluster in &clusters {
            // Split large clusters
            let chunks: Vec<&[Uuid]> = cluster
                .order_ids
                .chunks(self.max_orders_per_cluster)
                .collect();

            for chunk in chunks {
                total_clusters += 1;

                let batch_cluster = BatchCluster {
                    id: Uuid::now_v7(),
                    batch_run_id: run.id,
                    cluster_label: cluster.label,
                    driver_id: None,
                    centroid: Some(cluster.centroid),
                    order_count: chunk.len() as i32,
                    total_distance_meters: None,
                    created_at: now,
                };
                let created_cluster = self.batch_repo.create_cluster(&batch_cluster).await?;

                for order_id in chunk {
                    self.order_repo
                        .assign_to_batch_cluster(*order_id, created_cluster.id)
                        .await?;
                    self.order_repo
                        .update_status(*order_id, OrderStatus::Dispatching, None)
                        .await?;
                }

                // Assign nearest available driver to cluster centroid
                let nearby = self
                    .geospatial
                    .find_nearby_drivers(cluster.centroid, 50_000.0, 1)
                    .await?;

                if let Some(driver) = nearby.first() {
                    total_drivers += 1;
                    for order_id in chunk {
                        self.order_repo
                            .update_status(*order_id, OrderStatus::Assigned, Some(driver.driver_id))
                            .await?;
                    }
                }
            }
        }

        self.batch_repo
            .complete_run(run.id, total_clusters, total_drivers)
            .await?;

        tracing::info!(
            run_id = %run.id,
            orders = pending.len(),
            clusters = total_clusters,
            drivers = total_drivers,
            "Batch run completed"
        );

        Ok(run.id)
    }
}
