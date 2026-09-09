-- Lap table

ALTER TABLE ADDITIONAL_DATA DROP COLUMN laps;

CREATE TABLE LAP (
    session INTEGER NOT NULL,
    idx INTEGER NOT NULL,
    start_latitude INTEGER,
    start_longitude INTEGER,

    PRIMARY KEY (session, idx),
    FOREIGN KEY (session) REFERENCES SESSION(date) ON DELETE CASCADE
);
CREATE INDEX LAP_SESSION ON LAP(session);