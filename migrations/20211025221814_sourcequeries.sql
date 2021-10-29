CREATE TABLE sourcequeries (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sourcequery_type TEXT NOT NULL,
  sourcequery_data jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL
);

SELECT sqlx_manage_updated_at('sourcequeries');

