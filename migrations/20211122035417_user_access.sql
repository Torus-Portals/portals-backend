CREATE TABLE user_access (
  PRIMARY KEY (user_id, object_id, object_type),
  user_id UUID NOT NULL,
  object_id UUID NOT NULL,
  object_type TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL
)