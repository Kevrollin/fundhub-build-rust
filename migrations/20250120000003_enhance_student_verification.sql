-- Enhanced Student Verification System
-- This migration adds the missing fields for the comprehensive student verification flow

-- Add missing fields to student_verifications table
ALTER TABLE student_verifications 
    ADD COLUMN IF NOT EXISTS full_name VARCHAR(255),
    ADD COLUMN IF NOT EXISTS school_name VARCHAR(255),
    ADD COLUMN IF NOT EXISTS student_bio TEXT,
    ADD COLUMN IF NOT EXISTS motivation_text TEXT,
    ADD COLUMN IF NOT EXISTS submitted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;

-- Add verification status tracking to users table
ALTER TABLE users 
    ADD COLUMN IF NOT EXISTS verification_status VARCHAR(50) DEFAULT 'not_applied',
    ADD COLUMN IF NOT EXISTS verification_submitted_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS verification_approved_at TIMESTAMP WITH TIME ZONE;

-- Create student_profiles table for additional student information
CREATE TABLE IF NOT EXISTS student_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    full_name VARCHAR(255) NOT NULL,
    school_name VARCHAR(255) NOT NULL,
    school_email VARCHAR(255) NOT NULL,
    student_bio TEXT,
    motivation_text TEXT,
    profile_picture_url VARCHAR(500),
    linkedin_url VARCHAR(500),
    github_url VARCHAR(500),
    portfolio_url VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create verification_history table for tracking verification attempts
CREATE TABLE IF NOT EXISTS verification_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    verification_id UUID REFERENCES student_verifications(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL,
    admin_message TEXT,
    admin_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_verification_status ON users(verification_status);
CREATE INDEX IF NOT EXISTS idx_users_verification_submitted_at ON users(verification_submitted_at);
CREATE INDEX IF NOT EXISTS idx_student_profiles_user_id ON student_profiles(user_id);
CREATE INDEX IF NOT EXISTS idx_verification_history_user_id ON verification_history(user_id);
CREATE INDEX IF NOT EXISTS idx_verification_history_status ON verification_history(status);

-- Update existing student_verifications to have proper status values
UPDATE student_verifications 
SET status = 'pending' 
WHERE status IS NULL OR status = '';

-- Update users table to reflect current verification status
UPDATE users 
SET verification_status = 'verified',
    verification_approved_at = CURRENT_TIMESTAMP
WHERE id IN (
    SELECT user_id FROM student_verifications 
    WHERE status = 'verified'
);

UPDATE users 
SET verification_status = 'pending',
    verification_submitted_at = CURRENT_TIMESTAMP
WHERE id IN (
    SELECT user_id FROM student_verifications 
    WHERE status = 'pending'
);

UPDATE users 
SET verification_status = 'rejected'
WHERE id IN (
    SELECT user_id FROM student_verifications 
    WHERE status = 'rejected'
);
