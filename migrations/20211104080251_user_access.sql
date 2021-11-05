CREATE TABLE user_access (
  PRIMARY KEY (user_id, object_id),
  user_id UUID NOT NULL,
  object_type TEXT NOT NULL,
  object_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL,

  CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id)
);
