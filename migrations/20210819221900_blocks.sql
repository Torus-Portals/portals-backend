CREATE TABLE blocks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  block_type TEXT NOT NULL,
  portal_id UUID NOT NULL,
  portal_view_id UUID NOT NULL,
  egress TEXT NOT NULL,
  bbox INT[] NOT NULL DEFAULT '{0, 0, 0, 0}'::INT[],
  data jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL
);

SELECT sqlx_manage_updated_at('blocks');