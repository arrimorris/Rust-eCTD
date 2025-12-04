-- Add Application and Applicant metadata columns to the Submission Unit
-- This makes the schema compatible with the 'init' command
ALTER TABLE submission_units
ADD COLUMN IF NOT EXISTS application_id_uuid UUID NOT NULL DEFAULT gen_random_uuid(),
ADD COLUMN IF NOT EXISTS application_code VARCHAR(64) NOT NULL DEFAULT 'nda',
ADD COLUMN IF NOT EXISTS application_number VARCHAR(64) NOT NULL DEFAULT '000000',
ADD COLUMN IF NOT EXISTS applicant_name VARCHAR(255) NOT NULL DEFAULT 'Unknown',
ADD COLUMN IF NOT EXISTS submission_code VARCHAR(64) NOT NULL DEFAULT 'seq-0001';
