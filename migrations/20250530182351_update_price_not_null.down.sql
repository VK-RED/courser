-- Add down migration script here
ALTER TABLE "course_table"
ALTER COLUMN price SET NULL;