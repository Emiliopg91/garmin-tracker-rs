--- Settings table and location column

CREATE TABLE IF NOT EXISTS SETTINGS(
    name TEXT NOT NULL,
    value TEXT NOT NULL,

    PRIMARY KEY(name)
);

INSERT INTO SETTINGS VALUES('auto_sync', 'true');
INSERT INTO SETTINGS VALUES('distance_unit', 'Kilometers');
INSERT INTO SETTINGS VALUES('weight_unit', 'Kilograms');

ALTER TABLE GPS_COORDINATES ADD COLUMN location TEXT DEFAULT NULL;
