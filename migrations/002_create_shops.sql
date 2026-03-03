CREATE TABLE IF NOT EXISTS shops (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    phone VARCHAR(20) NOT NULL UNIQUE,
    api_key_hash VARCHAR(255) NOT NULL,
    location GEOMETRY(Point, 4326) NOT NULL,
    address TEXT NOT NULL,
    webhook_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shops_location ON shops USING GIST (location);
CREATE INDEX IF NOT EXISTS idx_shops_api_key_hash ON shops (api_key_hash);
