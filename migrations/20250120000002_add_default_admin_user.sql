-- Add default admin user
-- This migration creates a default admin user for platform management

-- Insert the default admin user
-- Password: Kevdev@2025 (will be hashed with Argon2)
INSERT INTO users (
    id,
    username,
    email,
    password_hash,
    role,
    base_role,
    is_verified,
    status,
    created_at
) VALUES (
    '00000000-0000-0000-0000-000000000001',
    'fundhubadmin',
    'fundhubadmin@2025',
    '$argon2id$v=19$m=65536,t=3,p=4$e8bb16cfe82738731c760dd66686564d$3RQSWcr5O9UDB1erQ96UAtWoeloyrJJ/+au4PCzQjDk=',
    'admin',
    'admin',
    true,
    'active',
    NOW()
) ON CONFLICT (email) DO NOTHING;

-- Create an index for admin users if it doesn't exist
CREATE INDEX IF NOT EXISTS idx_users_admin_role ON users(role) WHERE role = 'admin';
