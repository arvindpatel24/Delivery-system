use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::{GeospatialEngine, NearbyDriver, SpatialCluster};
use delivery_domain::value_objects::Location;

pub struct PostgisEngine {
    pub pool: PgPool,
}

#[derive(sqlx::FromRow)]
struct NearbyDriverRow {
    driver_id: Uuid,
    distance_meters: f64,
    lat: f64,
    lng: f64,
}

#[derive(sqlx::FromRow)]
struct ClusterRow {
    order_id: Uuid,
    cluster_label: i32,
    centroid_lat: f64,
    centroid_lng: f64,
}

#[async_trait]
impl GeospatialEngine for PostgisEngine {
    async fn find_nearby_drivers(
        &self,
        pickup: Location,
        radius_meters: f64,
        limit: i64,
    ) -> DomainResult<Vec<NearbyDriver>> {
        let rows = sqlx::query_as::<_, NearbyDriverRow>(
            r#"
            SELECT
                d.id as driver_id,
                ST_Distance(d.current_location::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) as distance_meters,
                ST_Y(d.current_location) as lat,
                ST_X(d.current_location) as lng
            FROM drivers d
            WHERE d.is_available = TRUE
                AND d.is_active = TRUE
                AND d.current_location IS NOT NULL
                AND ST_DWithin(
                    d.current_location::geography,
                    ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
                    $3
                )
            ORDER BY distance_meters ASC
            LIMIT $4
            "#,
        )
        .bind(pickup.longitude)
        .bind(pickup.latitude)
        .bind(radius_meters)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| NearbyDriver {
                driver_id: r.driver_id,
                distance_meters: r.distance_meters,
                location: Location {
                    latitude: r.lat,
                    longitude: r.lng,
                },
            })
            .collect())
    }

    async fn compute_distance(&self, from: Location, to: Location) -> DomainResult<f64> {
        let row = sqlx::query_as::<_, (f64,)>(
            r#"
            SELECT ST_Distance(
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
                ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography
            )
            "#,
        )
        .bind(from.longitude)
        .bind(from.latitude)
        .bind(to.longitude)
        .bind(to.latitude)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.0)
    }

    async fn cluster_orders(
        &self,
        order_ids: &[Uuid],
        eps_meters: f64,
        min_points: i32,
    ) -> DomainResult<Vec<SpatialCluster>> {
        if order_ids.is_empty() {
            return Ok(vec![]);
        }

        // Use ST_ClusterDBSCAN with UTM projection for meter-based eps
        let rows = sqlx::query_as::<_, ClusterRow>(
            r#"
            WITH clustered AS (
                SELECT
                    o.id as order_id,
                    ST_ClusterDBSCAN(ST_Transform(o.dropoff_location, 32643), eps := $2, minpoints := $3)
                        OVER () as cluster_label,
                    o.dropoff_location
                FROM orders o
                WHERE o.id = ANY($1)
            )
            SELECT
                c.order_id,
                COALESCE(c.cluster_label, -1) as cluster_label,
                ST_Y(ST_Centroid(ST_Collect(c.dropoff_location) OVER (PARTITION BY c.cluster_label))) as centroid_lat,
                ST_X(ST_Centroid(ST_Collect(c.dropoff_location) OVER (PARTITION BY c.cluster_label))) as centroid_lng
            FROM clustered c
            WHERE c.cluster_label IS NOT NULL
            "#,
        )
        .bind(order_ids)
        .bind(eps_meters)
        .bind(min_points)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        // Group by cluster_label
        let mut cluster_map: std::collections::HashMap<i32, SpatialCluster> =
            std::collections::HashMap::new();

        for row in rows {
            let entry = cluster_map.entry(row.cluster_label).or_insert_with(|| {
                SpatialCluster {
                    label: row.cluster_label,
                    order_ids: vec![],
                    centroid: Location {
                        latitude: row.centroid_lat,
                        longitude: row.centroid_lng,
                    },
                }
            });
            entry.order_ids.push(row.order_id);
        }

        Ok(cluster_map.into_values().collect())
    }
}
