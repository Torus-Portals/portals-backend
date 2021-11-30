CREATE TABLE blocks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  project_id UUID NOT NULL,
  dashboard_id UUID NOT NULL,
  page_id UUID NOT NULL,
  block_type TEXT NOT NULL,
  block_data jsonb NOT NULL DEFAULT '{}'::jsonb,
  block_config jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL,
  deleted_at TIMESTAMPTZ DEFAULT NULL,
  deleted_by UUID DEFAULT NULL
);

SELECT sqlx_manage_updated_at('blocks');