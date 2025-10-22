-- Add role system and guest funding flow
-- This migration adds the new role hierarchy, guest funding, and related tables

-- Add new columns to users table for role system
ALTER TABLE users 
    ADD COLUMN IF NOT EXISTS base_role VARCHAR(50) DEFAULT 'base_user',
    ADD COLUMN IF NOT EXISTS is_verified BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS last_login TIMESTAMP WITH TIME ZONE;

-- Create guest_donations table for anonymous funding
CREATE TABLE IF NOT EXISTS guest_donations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guest_name VARCHAR(255) NOT NULL,
    guest_email VARCHAR(255) NOT NULL,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tx_hash VARCHAR(255),
    amount DECIMAL(20,8) NOT NULL,
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create student_verifications table (decoupled from students table)
CREATE TABLE IF NOT EXISTS student_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    school_email VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    admin_message TEXT,
    approved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add visibility column to projects
ALTER TABLE projects 
    ADD COLUMN IF NOT EXISTS visibility VARCHAR(50) DEFAULT 'public';

-- Create milestones table for smart disbursement
CREATE TABLE IF NOT EXISTS milestones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    target_amount DECIMAL(20,8) NOT NULL,
    released BOOLEAN DEFAULT FALSE,
    released_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create activity_logs table for admin oversight
CREATE TABLE IF NOT EXISTS activity_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(255) NOT NULL,
    target_id UUID,
    target_type VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add payment_method column to donations (if not exists)
ALTER TABLE donations 
    ADD COLUMN IF NOT EXISTS payment_method VARCHAR(50) DEFAULT 'stellar';

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_base_role ON users(base_role);
CREATE INDEX IF NOT EXISTS idx_users_is_verified ON users(is_verified);
CREATE INDEX IF NOT EXISTS idx_guest_donations_project_id ON guest_donations(project_id);
CREATE INDEX IF NOT EXISTS idx_guest_donations_tx_hash ON guest_donations(tx_hash);
CREATE INDEX IF NOT EXISTS idx_student_verifications_user_id ON student_verifications(user_id);
CREATE INDEX IF NOT EXISTS idx_student_verifications_status ON student_verifications(status);
CREATE INDEX IF NOT EXISTS idx_projects_visibility ON projects(visibility);
CREATE INDEX IF NOT EXISTS idx_milestones_project_id ON milestones(project_id);
CREATE INDEX IF NOT EXISTS idx_milestones_released ON milestones(released);
CREATE INDEX IF NOT EXISTS idx_activity_logs_user_id ON activity_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_activity_logs_target ON activity_logs(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_activity_logs_created_at ON activity_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_donations_payment_method ON donations(payment_method);

-- Update existing users to have proper base_role
UPDATE users SET base_role = 'base_user' WHERE base_role IS NULL;

-- Update existing students to be verified
UPDATE users 
SET is_verified = TRUE, base_role = 'student' 
WHERE id IN (SELECT user_id FROM students WHERE verification_status = 'verified');
