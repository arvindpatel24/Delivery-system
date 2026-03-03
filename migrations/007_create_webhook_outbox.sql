DO $$ BEGIN
    CREATE TYPE webhook_status AS ENUM ('pending', 'delivered', 'failed', 'dead');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS webhook_outbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES orders(id),
    shop_id UUID NOT NULL REFERENCES shops(id),
    webhook_url TEXT NOT NULL,
    payload JSONB NOT NULL,
    status webhook_status NOT NULL DEFAULT 'pending',
    attempts INT NOT NULL DEFAULT 0,
    last_error TEXT,
    next_retry_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivered_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_webhook_outbox_pending ON webhook_outbox (next_retry_at)
    WHERE status = 'pending' OR status = 'failed';
