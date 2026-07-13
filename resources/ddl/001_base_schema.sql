-- Initial scheme

CREATE TABLE IF NOT EXISTS EXERCISE(
    category TEXT NOT NULL,
    id INTEGER NOT NULL,
    name TEXT NOT NULL,

    PRIMARY KEY(category, id)
);
CREATE INDEX IF NOT EXISTS EXERCISE_ID_CAT ON EXERCISE(category, id);

CREATE TABLE IF NOT EXISTS SESSION(
    date INTEGER NOT NULL,
    workout TEXT NOT NULL,
    total_elapsed_time REAL NOT NULL,
    active_time REAL NOT NULL,
    total_calories INTEGER NOT NULL,
    metabolic_calories INTEGER NOT NULL,
    avg_heart_rate INTEGER NOT NULL,
    max_heart_rate INTEGER NOT NULL,

    PRIMARY KEY(date)
);
CREATE INDEX IF NOT EXISTS SESSION_WORKOUT ON SESSION(workout);
CREATE INDEX IF NOT EXISTS SESSION_DATE ON SESSION(date);

CREATE TABLE IF NOT EXISTS SERIE(
    session INTEGER NOT NULL,
    idx INTEGER NOT NULL,
    exercise_category TEXT NOT NULL,
    exercise_id INTEGER NOT NULL,
    reps INTEGER NOT NULL,
    weight REAL NOT NULL,
    pr BOOLEAN NOT NULL,

    PRIMARY KEY(session, idx),

    FOREIGN KEY(session) REFERENCES SESSION(date) ON DELETE CASCADE,
    FOREIGN KEY(exercise_category, exercise_id) REFERENCES EXERCISE(category, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS SERIE_ID ON SERIE(session, idx);
CREATE INDEX IF NOT EXISTS SERIE_EXERCISE ON SERIE(exercise_category, exercise_id);
