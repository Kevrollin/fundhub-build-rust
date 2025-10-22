-- Add payment provider integration tables
-- This migration adds tables for M-Pesa, Stripe, and other payment providers

-- Create payment_instructions table for storing payment initiation data
CREATE TABLE IF NOT EXISTS payment_instructions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id VARCHAR(255) NOT NULL UNIQUE,
    payment_method VARCHAR(50) NOT NULL,
    instructions JSONB NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add provider fields to donations table
ALTER TABLE donations 
    ADD COLUMN IF NOT EXISTS provider_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS provider_status VARCHAR(50),
    ADD COLUMN IF NOT EXISTS provider_raw JSONB;

-- Create refunds table for tracking refunds
CREATE TABLE IF NOT EXISTS refunds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    refund_id VARCHAR(255) NOT NULL UNIQUE,
    payment_id VARCHAR(255) NOT NULL,
    amount DECIMAL(20,8),
    reason TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    processed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create fiat_settlements table for reconciliation
CREATE TABLE IF NOT EXISTS fiat_settlements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id VARCHAR(255) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    fiat_amount DECIMAL(20,8) NOT NULL,
    fiat_currency VARCHAR(10) NOT NULL,
    xlm_amount DECIMAL(20,8) NOT NULL,
    exchange_rate DECIMAL(20,8) NOT NULL,
    tx_hash VARCHAR(255),
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create payment_reconciliation table for tracking reconciliation jobs
CREATE TABLE IF NOT EXISTS payment_reconciliation (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider VARCHAR(50) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    processed_count INTEGER DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_payment_instructions_payment_id ON payment_instructions(payment_id);
CREATE INDEX IF NOT EXISTS idx_payment_instructions_payment_method ON payment_instructions(payment_method);
CREATE INDEX IF NOT EXISTS idx_payment_instructions_expires_at ON payment_instructions(expires_at);
CREATE INDEX IF NOT EXISTS idx_donations_provider_id ON donations(provider_id);
CREATE INDEX IF NOT EXISTS idx_donations_provider_status ON donations(provider_status);
CREATE INDEX IF NOT EXISTS idx_refunds_payment_id ON refunds(payment_id);
CREATE INDEX IF NOT EXISTS idx_refunds_status ON refunds(status);
CREATE INDEX IF NOT EXISTS idx_fiat_settlements_payment_id ON fiat_settlements(payment_id);
CREATE INDEX IF NOT EXISTS idx_fiat_settlements_provider ON fiat_settlements(provider);
CREATE INDEX IF NOT EXISTS idx_fiat_settlements_status ON fiat_settlements(status);
CREATE INDEX IF NOT EXISTS idx_payment_reconciliation_provider ON payment_reconciliation(provider);
CREATE INDEX IF NOT EXISTS idx_payment_reconciliation_status ON payment_reconciliation(status);

-- Update existing donations to have default provider values
UPDATE donations 
SET provider_id = tx_hash, 
    provider_status = status, 
    provider_raw = '{}'::jsonb
WHERE provider_id IS NULL;
