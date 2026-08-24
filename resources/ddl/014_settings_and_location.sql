--- Settings table and location column

CREATE TABLE IF NOT EXISTS SETTINGS(
    name TEXT NOT NULL,
    value TEXT NOT NULL,

    PRIMARY KEY(name)
);

INSERT INTO SETTINGS VALUES ('auto_sync', 'true'), ('distance_unit', 'Kilometers'), ('start_boot', 'false'), ('weight_unit', 'Kilograms');
