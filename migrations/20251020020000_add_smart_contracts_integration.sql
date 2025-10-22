-- Add smart contracts integration tables
-- This migration adds tables for Soroban smart contract integration

-- Create contracts table to store deployed contract addresses
CREATE TABLE IF NOT EXISTS contracts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    address VARCHAR(255) NOT NULL UNIQUE,
    network VARCHAR(50) NOT NULL DEFAULT 'testnet',
    deployed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add contract_address column to projects table
ALTER TABLE projects 
    ADD COLUMN IF NOT EXISTS contract_address VARCHAR(255);

-- Create contract_milestones table for on-chain milestone tracking
CREATE TABLE IF NOT EXISTS contract_milestones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    milestone_id VARCHAR(255) NOT NULL, -- On-chain milestone ID
    amount_stroops BIGINT NOT NULL,
    proof_required BOOLEAN DEFAULT FALSE,
    released BOOLEAN DEFAULT FALSE,
    released_at TIMESTAMP WITH TIME ZONE,
    recipient_address VARCHAR(255), -- Stellar address
    attestation_signature TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create contract_deposits table for tracking on-chain deposits
CREATE TABLE IF NOT EXISTS contract_deposits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    donor_address VARCHAR(255) NOT NULL,
    amount_stroops BIGINT NOT NULL,
    memo TEXT,
    tx_hash VARCHAR(255) NOT NULL UNIQUE,
    block_number BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create contract_releases table for tracking milestone releases
CREATE TABLE IF NOT EXISTS contract_releases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    milestone_id VARCHAR(255) NOT NULL,
    recipient_address VARCHAR(255) NOT NULL,
    amount_stroops BIGINT NOT NULL,
    tx_hash VARCHAR(255) NOT NULL UNIQUE,
    attestation_signature TEXT,
    block_number BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_contracts_name ON contracts(name);
CREATE INDEX IF NOT EXISTS idx_contracts_network ON contracts(network);
CREATE INDEX IF NOT EXISTS idx_projects_contract_address ON projects(contract_address);
CREATE INDEX IF NOT EXISTS idx_contract_milestones_project_id ON contract_milestones(project_id);
CREATE INDEX IF NOT EXISTS idx_contract_milestones_milestone_id ON contract_milestones(milestone_id);
CREATE INDEX IF NOT EXISTS idx_contract_milestones_released ON contract_milestones(released);
CREATE INDEX IF NOT EXISTS idx_contract_deposits_project_id ON contract_deposits(project_id);
CREATE INDEX IF NOT EXISTS idx_contract_deposits_tx_hash ON contract_deposits(tx_hash);
CREATE INDEX IF NOT EXISTS idx_contract_deposits_donor ON contract_deposits(donor_address);
CREATE INDEX IF NOT EXISTS idx_contract_releases_project_id ON contract_releases(project_id);
CREATE INDEX IF NOT EXISTS idx_contract_releases_tx_hash ON contract_releases(tx_hash);
CREATE INDEX IF NOT EXISTS idx_contract_releases_recipient ON contract_releases(recipient_address);

-- Insert default contract records (will be updated after deployment)
INSERT INTO contracts (name, address, network) VALUES 
    ('project_registry', 'PLACEHOLDER_PROJECT_REGISTRY_ADDRESS', 'testnet'),
    ('funding_escrow', 'PLACEHOLDER_FUNDING_ESCROW_ADDRESS', 'testnet'),
    ('milestone_manager', 'PLACEHOLDER_MILESTONE_MANAGER_ADDRESS', 'testnet')
ON CONFLICT (name) DO NOTHING;
