CREATE TABLE portals (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  org UUID NOT NULL,
  owners UUID[] NOT NULL DEFAULT '{}'::UUID[],
  vendors UUID[] NOT NULL DEFAULT '{}'::UUID[],
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL
);

SELECT sqlx_manage_updated_at('portals');
