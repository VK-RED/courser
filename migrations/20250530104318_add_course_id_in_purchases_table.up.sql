-- Add up migration script here
ALTER TABLE "purchases_table"
ADD COLUMN course_id uuid NOT NULL;