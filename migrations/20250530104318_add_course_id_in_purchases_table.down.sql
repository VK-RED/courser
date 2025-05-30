-- Add down migration script here
ALTER TABLE "purchases_table"
DROP COLUMN IF EXISTS course_id;