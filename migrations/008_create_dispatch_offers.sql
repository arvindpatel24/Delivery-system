DO $$ BEGIN
    CREATE TYPE offer_status AS ENUM ('pending', 'accepted', 'rejected', 'expired');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS driver_dispatch_offers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES orders(id),
    driver_id UUID NOT NULL REFERENCES drivers(id),
    status offer_status NOT NULL DEFAULT 'pending',
    distance_to_pickup_meters DOUBLE PRECISION NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    responded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dispatch_offers_order ON driver_dispatch_offers (order_id);
CREATE INDEX IF NOT EXISTS idx_dispatch_offers_driver_pending ON driver_dispatch_offers (driver_id, status)
    WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_dispatch_offers_expiry ON driver_dispatch_offers (expires_at)
    WHERE status = 'pending';
