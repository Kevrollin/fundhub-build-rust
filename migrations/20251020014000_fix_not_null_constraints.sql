-- Fix NOT NULL constraints for columns that should not be nullable

-- Update existing NULL values first
UPDATE projects SET status = 'pending_review' WHERE status IS NULL;
UPDATE projects SET tags = '{}' WHERE tags IS NULL;

-- Add NOT NULL constraints
ALTER TABLE projects ALTER COLUMN status SET NOT NULL;
ALTER TABLE projects ALTER COLUMN status SET DEFAULT 'pending_review';
ALTER TABLE projects ALTER COLUMN tags SET NOT NULL;
ALTER TABLE projects ALTER COLUMN tags SET DEFAULT '{}';
ALTER TABLE projects ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE projects ALTER COLUMN created_at SET DEFAULT NOW();

