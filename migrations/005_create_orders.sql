DO $$ BEGIN
    CREATE TYPE order_status AS ENUM (
        'pending', 'dispatching', 'assigned', 'picked_up',
        'in_transit', 'delivered', 'cancelled', 'failed'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE routing_mode AS ENUM ('instant', 'batched');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shop_id UUID NOT NULL REFERENCES shops(id),
    driver_id UUID REFERENCES drivers(id),
    status order_status NOT NULL DEFAULT 'pending',
    routing_mode routing_mode NOT NULL,
    pickup_address TEXT NOT NULL,
    pickup_location GEOMETRY(Point, 4326) NOT NULL,
    dropoff_address TEXT NOT NULL,
    dropoff_location GEOMETRY(Point, 4326) NOT NULL,
    distance_meters DOUBLE PRECISION NOT NULL,
    customer_name VARCHAR(255),
    customer_phone VARCHAR(20),
    package_description TEXT,
    estimated_delivery_at TIMESTAMPTZ,
    picked_up_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    batch_cluster_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_pickup ON orders USING GIST (pickup_location);
CREATE INDEX IF NOT EXISTS idx_orders_dropoff ON orders USING GIST (dropoff_location);
CREATE INDEX IF NOT EXISTS idx_orders_shop ON orders (shop_id);
CREATE INDEX IF NOT EXISTS idx_orders_driver ON orders (driver_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);
CREATE INDEX IF NOT EXISTS idx_orders_pending_batched ON orders (routing_mode, status) WHERE status = 'pending' AND routing_mode = 'batched';
