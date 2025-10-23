-- Create wallets table for storing user public keys
CREATE TABLE IF NOT EXISTS wallets (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    public_key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Create transactions table for tracking payments between wallets
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    sender_address TEXT NOT NULL,
    receiver_address TEXT NOT NULL,
    amount DECIMAL(20,7) NOT NULL,
    asset TEXT DEFAULT 'XLM',
    tx_hash TEXT UNIQUE NOT NULL,
    memo TEXT,
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'failed')),
    created_at TIMESTAMP DEFAULT NOW(),
    confirmed_at TIMESTAMP
);

-- Create platform_funds table for tracking total support received
CREATE TABLE IF NOT EXISTS platform_funds (
    id SERIAL PRIMARY KEY,
    donor_address TEXT,
    amount DECIMAL(20,7) NOT NULL,
    tx_hash TEXT UNIQUE NOT NULL,
    memo TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallets_public_key ON wallets(public_key);
CREATE INDEX IF NOT EXISTS idx_transactions_sender ON transactions(sender_address);
CREATE INDEX IF NOT EXISTS idx_transactions_receiver ON transactions(receiver_address);
CREATE INDEX IF NOT EXISTS idx_transactions_hash ON transactions(tx_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_platform_funds_created_at ON platform_funds(created_at);

-- Add comments for documentation
COMMENT ON TABLE wallets IS 'Stores user wallet public keys for Stellar integration';
COMMENT ON TABLE transactions IS 'Tracks all Stellar transactions between wallets';
COMMENT ON TABLE platform_funds IS 'Tracks donations to the platform wallet';
