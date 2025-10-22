-- Add missing columns to existing tables

-- Add confirmed_at to donations table
ALTER TABLE donations ADD COLUMN IF NOT EXISTS confirmed_at TIMESTAMP WITH TIME ZONE;

-- Add memo column to donations for stellar memo matching
ALTER TABLE donations ADD COLUMN IF NOT EXISTS memo VARCHAR(255);

-- Make tx_hash nullable for pending donations
ALTER TABLE donations ALTER COLUMN tx_hash DROP NOT NULL;

-- Add contract_address to projects
ALTER TABLE projects ADD COLUMN IF NOT EXISTS contract_address VARCHAR(255);

-- Add status to projects for approval workflow
ALTER TABLE projects ADD COLUMN IF NOT EXISTS status VARCHAR(50) NOT NULL DEFAULT 'pending_review';

-- Add admission_number to students
ALTER TABLE students ADD COLUMN IF NOT EXISTS admission_number VARCHAR(255);

-- Add verified_at timestamp to students
ALTER TABLE students ADD COLUMN IF NOT EXISTS verified_at TIMESTAMP WITH TIME ZONE;

-- Add verified_by admin reference
ALTER TABLE students ADD COLUMN IF NOT EXISTS verified_by UUID REFERENCES users(id);

-- Create refresh_tokens table
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create email_verification_tokens table
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    verified_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create files table for document storage
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
    entity_type VARCHAR(50), -- 'student_verification', 'project_media', etc
    entity_id UUID,
    path TEXT NOT NULL,
    filename VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100),
    size_bytes BIGINT,
    checksum VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create project_milestones table
CREATE TABLE IF NOT EXISTS project_milestones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    amount_stroops BIGINT NOT NULL,
    proof_type VARCHAR(50) DEFAULT 'upload', -- 'upload', 'link', 'manual'
    position INTEGER NOT NULL,
    status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'in_progress', 'completed', 'claimed'
    proof_url TEXT,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create onchain_transactions table for indexer
CREATE TABLE IF NOT EXISTS onchain_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tx_hash VARCHAR(255) NOT NULL UNIQUE,
    source_account VARCHAR(255),
    destination_account VARCHAR(255),
    amount_stroops BIGINT,
    amount_xlm DECIMAL(20, 8),
    memo TEXT,
    memo_type VARCHAR(50),
    ledger INTEGER,
    operation_type VARCHAR(50),
    successful BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    indexed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create verification_documents junction table
CREATE TABLE IF NOT EXISTS verification_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    student_id UUID NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    document_type VARCHAR(100) NOT NULL, -- 'id_card', 'admission_letter', 'student_id', etc
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(student_id, file_id)
);

-- Create project_updates table
CREATE TABLE IF NOT EXISTS project_updates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_email_verification_user_id ON email_verification_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_email_verification_token ON email_verification_tokens(token);
CREATE INDEX IF NOT EXISTS idx_files_owner_id ON files(owner_id);
CREATE INDEX IF NOT EXISTS idx_files_entity ON files(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_project_milestones_project_id ON project_milestones(project_id);
CREATE INDEX IF NOT EXISTS idx_onchain_tx_hash ON onchain_transactions(tx_hash);
CREATE INDEX IF NOT EXISTS idx_onchain_destination ON onchain_transactions(destination_account);
CREATE INDEX IF NOT EXISTS idx_onchain_memo ON onchain_transactions(memo);
CREATE INDEX IF NOT EXISTS idx_verification_docs_student ON verification_documents(student_id);
CREATE INDEX IF NOT EXISTS idx_project_updates_project ON project_updates(project_id);
CREATE INDEX IF NOT EXISTS idx_donations_memo ON donations(memo);
CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);

