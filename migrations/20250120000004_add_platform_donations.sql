-- Add support for platform donations (donations without project_id)
-- This allows donations to go directly to the platform wallet

-- Make project_id nullable in donations table
ALTER TABLE donations ALTER COLUMN project_id DROP NOT NULL;

-- Add a new column to track donation type
ALTER TABLE donations ADD COLUMN donation_type VARCHAR(50) DEFAULT 'project';

-- Update existing donations to have 'project' type
UPDATE donations SET donation_type = 'project' WHERE donation_type IS NULL;

-- Add index for platform donations
CREATE INDEX idx_donations_platform ON donations(donation_type) WHERE donation_type = 'platform';

-- Add index for project donations
CREATE INDEX idx_donations_project ON donations(project_id) WHERE project_id IS NOT NULL;
