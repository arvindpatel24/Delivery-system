use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainResult;
use crate::value_objects::Location;

#[derive(Debug, Clone)]
pub struct NearbyDriver {
    pub driver_id: Uuid,
    pub distance_meters: f64,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct SpatialCluster {
    pub label: i32,
    pub order_ids: Vec<Uuid>,
    pub centroid: Location,
}

#[async_trait]
pub trait GeospatialEngine: Send + Sync {
    async fn find_nearby_drivers(
        &self,
        pickup: Location,
        radius_meters: f64,
        limit: i64,
    ) -> DomainResult<Vec<NearbyDriver>>;

    async fn compute_distance(
        &self,
        from: Location,
        to: Location,
    ) -> DomainResult<f64>;

    async fn cluster_orders(
        &self,
        order_ids: &[Uuid],
        eps_meters: f64,
        min_points: i32,
    ) -> DomainResult<Vec<SpatialCluster>>;
}
