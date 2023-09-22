CREATE TABLE IF NOT EXISTS escalonjobs ();

ALTER TABLE escalonjobs
  ADD COLUMN id       UUID NOT NULL PRIMARY KEY,
  ADD COLUMN status   VARCHAR(20) NOT NULL,
  ADD COLUMN schedule VARCHAR(20) NOT NULL,
  ADD COLUMN since    TIMESTAMP WITH TIME ZONE,
  ADD COLUMN until    TIMESTAMP WITH TIME ZONE;

ALTER TABLE escalonjobs ADD CONSTRAINT check_escalonjobs_state CHECK
  (status IN ('scheduled', 'running', 'done', 'failed'));
