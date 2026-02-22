-- Add delete audit metadata to users for admin soft-delete actions
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS delete_reason TEXT;

CREATE INDEX IF NOT EXISTS idx_users_deleted_by ON users(deleted_by);
