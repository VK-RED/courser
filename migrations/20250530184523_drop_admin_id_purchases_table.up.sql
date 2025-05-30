-- Add up migration script here
ALTER TABLE "purchases_table"
DROP COLUMN IF EXISTS admin_id;