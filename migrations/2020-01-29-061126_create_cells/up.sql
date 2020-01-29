CREATE TABLE cells (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  data jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL
);

SELECT diesel_manage_updated_at('cells');
