CREATE TABLE IF NOT EXISTS drivers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    phone VARCHAR(20) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    vehicle_type VARCHAR(50) NOT NULL DEFAULT 'motorcycle',
    current_location GEOMETRY(Point, 4326),
    is_available BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_drivers_current_location ON drivers USING GIST (current_location);
CREATE INDEX IF NOT EXISTS idx_drivers_available ON drivers (is_available) WHERE is_available = TRUE;
