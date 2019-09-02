CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  auth0id VARCHAR(64) NOT NULL default '',
  name VARCHAR(64) NOT NULL,
  nickname VARCHAR(64) NOT NULL,
  email VARCHAR(64) NOT NULL,
  orgs Int[] NOT NULL default '{}'::Int[]
)