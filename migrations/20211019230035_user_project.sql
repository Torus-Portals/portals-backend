CREATE TABLE user_project (
  PRIMARY KEY (user_id, project_id),
  user_id UUID NOT NULL,
  project_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_by UUID NOT NULL,
  CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id),
  CONSTRAINT fk_project FOREIGN KEY(project_id) REFERENCES projects(id)
);