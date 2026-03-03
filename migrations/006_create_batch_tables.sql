CREATE TABLE IF NOT EXISTS batch_runs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    scheduled_hour INT NOT NULL,
    total_orders INT NOT NULL DEFAULT 0,
    total_clusters INT NOT NULL DEFAULT 0,
    total_drivers_assigned INT NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS batch_clusters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_run_id UUID NOT NULL REFERENCES batch_runs(id),
    cluster_label INT NOT NULL,
    driver_id UUID REFERENCES drivers(id),
    centroid GEOMETRY(Point, 4326),
    route_geometry GEOMETRY(LineString, 4326),
    order_count INT NOT NULL DEFAULT 0,
    total_distance_meters DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_batch_clusters_run ON batch_clusters (batch_run_id);
