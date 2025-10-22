-- Campaign distributions table
CREATE TABLE IF NOT EXISTS campaign_distributions (
    id UUID PRIMARY KEY,
    campaign_id UUID NOT NULL REFERENCES campaigns(id),
    recipient_id UUID NOT NULL REFERENCES students(id),
    amount DOUBLE PRECISION NOT NULL,
    tx_hash TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Daily analytics table
CREATE TABLE IF NOT EXISTS daily_analytics (
    date DATE NOT NULL,
    metric TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (date, metric)
);

-- Weekly analytics table
CREATE TABLE IF NOT EXISTS weekly_analytics (
    week_start DATE NOT NULL,
    metric TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (week_start, metric)
);

-- Add updated_at column to campaigns table
ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_campaign_distributions_campaign_id ON campaign_distributions(campaign_id);
CREATE INDEX IF NOT EXISTS idx_campaign_distributions_recipient_id ON campaign_distributions(recipient_id);
CREATE INDEX IF NOT EXISTS idx_daily_analytics_date ON daily_analytics(date);
CREATE INDEX IF NOT EXISTS idx_weekly_analytics_week_start ON weekly_analytics(week_start);
CREATE INDEX IF NOT EXISTS idx_analytics_summary_entity ON analytics_summary(entity_type, entity_id);
