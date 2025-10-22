-- Campaigns table
CREATE TABLE IF NOT EXISTS campaigns (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    criteria TEXT NOT NULL,
    reward_pool_xlm DOUBLE PRECISION NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Analytics summary table
CREATE TABLE IF NOT EXISTS analytics_summary (
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    metric TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (entity_type, entity_id, metric)
);
