CREATE TABLE IF NOT EXISTS driver_locations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    driver_id UUID NOT NULL REFERENCES drivers(id),
    location GEOMETRY(Point, 4326) NOT NULL,
    accuracy_meters DOUBLE PRECISION,
    speed_kmh DOUBLE PRECISION,
    heading DOUBLE PRECISION,
    is_offline_sync BOOLEAN NOT NULL DEFAULT FALSE,
    recorded_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_driver_locations_gist ON driver_locations USING GIST (location);
CREATE INDEX IF NOT EXISTS idx_driver_locations_driver_time ON driver_locations (driver_id, recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_driver_locations_cleanup ON driver_locations (created_at);
