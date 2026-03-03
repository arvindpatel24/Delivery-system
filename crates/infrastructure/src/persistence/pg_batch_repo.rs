use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use delivery_domain::entities::batch::{BatchCluster, BatchRun};
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::BatchRepository;

use super::models::{BatchClusterRow, BatchRunRow};

pub struct PgBatchRepo {
    pub pool: PgPool,
}

#[async_trait]
impl BatchRepository for PgBatchRepo {
    async fn create_run(&self, run: &BatchRun) -> DomainResult<BatchRun> {
        let row = sqlx::query_as::<_, BatchRunRow>(
            r#"
            INSERT INTO batch_runs (id, scheduled_hour, total_orders, started_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, scheduled_hour, total_orders, total_clusters, total_drivers_assigned, started_at, completed_at
            "#,
        )
        .bind(run.id)
        .bind(run.scheduled_hour)
        .bind(run.total_orders)
        .bind(run.started_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.into())
    }

    async fn complete_run(&self, id: Uuid, clusters: i32, drivers: i32) -> DomainResult<()> {
        sqlx::query(
            r#"
            UPDATE batch_runs SET total_clusters = $2, total_drivers_assigned = $3, completed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(clusters)
        .bind(drivers)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn create_cluster(&self, cluster: &BatchCluster) -> DomainResult<BatchCluster> {
        let (centroid_lng, centroid_lat) = cluster
            .centroid
            .map(|c| (Some(c.longitude), Some(c.latitude)))
            .unwrap_or((None, None));

        let row = sqlx::query_as::<_, BatchClusterRow>(
            r#"
            INSERT INTO batch_clusters (id, batch_run_id, cluster_label, driver_id, centroid, order_count, total_distance_meters, created_at)
            VALUES ($1, $2, $3, $4,
                CASE WHEN $5::float8 IS NOT NULL THEN ST_SetSRID(ST_MakePoint($5, $6), 4326) ELSE NULL END,
                $7, $8, $9)
            RETURNING id, batch_run_id, cluster_label, driver_id,
                ST_Y(centroid) as centroid_lat, ST_X(centroid) as centroid_lng,
                order_count, total_distance_meters, created_at
            "#,
        )
        .bind(cluster.id)
        .bind(cluster.batch_run_id)
        .bind(cluster.cluster_label)
        .bind(cluster.driver_id)
        .bind(centroid_lng)
        .bind(centroid_lat)
        .bind(cluster.order_count)
        .bind(cluster.total_distance_meters)
        .bind(cluster.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.into())
    }
}
