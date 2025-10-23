-- Make wallets flexible to support both regular users and students
-- Add user_id column to wallets table to support direct user-wallet relationships

-- Add user_id column to wallets table
ALTER TABLE wallets ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id);

-- Update existing wallets to have user_id from their student relationship
UPDATE wallets 
SET user_id = s.user_id 
FROM students s 
WHERE wallets.student_id = s.id;

-- Make user_id NOT NULL after populating it
ALTER TABLE wallets ALTER COLUMN user_id SET NOT NULL;

-- Make student_id nullable to support direct user-wallet relationships
ALTER TABLE wallets ALTER COLUMN student_id DROP NOT NULL;

-- Add index for user_id lookups
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);

-- Add comment to clarify the flexible relationship
COMMENT ON TABLE wallets IS 'Wallets can be associated with either students (via student_id) or regular users (via user_id)';
