-- Adds verification_message and verified_at to students table
ALTER TABLE students
    ADD COLUMN IF NOT EXISTS verification_message TEXT;
ALTER TABLE students
    ADD COLUMN IF NOT EXISTS verified_at TIMESTAMP WITH TIME ZONE;
