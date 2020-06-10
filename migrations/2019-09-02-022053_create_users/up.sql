

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  auth0id TEXT NOT NULL DEFAULT '',
  name TEXT NOT NULL,
  nickname TEXT NOT NULL,
  email TEXT NOT NULL,
  status TEXT NOT NULL,
  -- Maybe orgs should have a list of users?
  orgs UUID[] NOT NULL DEFAULT '{}'::UUID[],
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL
);

SELECT diesel_manage_updated_at('users');