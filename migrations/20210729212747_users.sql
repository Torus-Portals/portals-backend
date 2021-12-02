CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  auth0id TEXT NOT NULL DEFAULT '',
  name TEXT NOT NULL,
  nickname TEXT NOT NULL,
  email TEXT NOT NULL,
  user_status jsonb NOT NULL DEFAULT '{}'::jsonb,
  org_ids UUID[] NOT NULL DEFAULT '{}'::UUID[],
  role_ids UUID[] NOT NULL DEFAULT '{}'::UUID[],
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL
);

SELECT sqlx_manage_updated_at('users');