-- Add down migration script here
ALTER TABLE "purchases_table"
ADD COLUMN IF NOT EXISTS admin_id uuid NOT NULL;