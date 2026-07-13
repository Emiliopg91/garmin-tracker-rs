-- Adding user profile table

CREATE TABLE IF NOT EXISTS USER(
    date INTEGER NOT NULL,
    weight REAL NOT NULL,
    fat_ratio REAL NOT NULL,
    lean_mass REAL NOT NULL,
    water_ratio REAL NOT NULL,

    PRIMARY KEY(date)
);
CREATE INDEX IF NOT EXISTS USER_DATE ON USER(date);