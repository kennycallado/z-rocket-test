CREATE TABLE IF NOT EXISTS escalonjobs ();

ALTER TABLE escalonjobs
  ADD COLUMN id UUID NOT NULL PRIMARY KEY,
  ADD COLUMN schedule VARCHAR(20) NOT NULL,
  ADD COLUMN since TIMESTAMP,
  ADD COLUMN until TIMESTAMP;
