-- Your SQL goes here
CREATE TABLE Users (
  id SERIAL PRIMARY KEY,
  username VARCHAR(64) NOT NULL,
  firstname VARCHAR(64) NOT NULL,
  lastname VARCHAR(64) NOT NULL,
  email VARCHAR(64) NOT NULL,
  orgs Int[] NOT NULL default '{}'::Int[]
)