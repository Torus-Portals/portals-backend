-- Add migration script here
CREATE TABLE dimensions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  portal_id UUID NOT NULL,
  name TEXT NOT NULL, 
  dimension_type TEXT NOT NULL,
  meta jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL
);

SELECT sqlx_manage_updated_at('dimensions');