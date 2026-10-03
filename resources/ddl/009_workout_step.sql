-- Create table for workout steps

CREATE TABLE WORKOUT_STEP (
    workout TEXT NOT NULL,
    idx INTEGER NOT NULL,
    kind TEXT NOT NULL,
    exercise_category INTEGER,
    exercise_id INTEGER,
    reps INTEGER,
    weight REAL,
    time INTEGER,
    begin_idx INTEGER,

    PRIMARY KEY(workout, idx),
    FOREIGN KEY (workout) REFERENCES WORKOUT(name) ON DELETE CASCADE,
    FOREIGN KEY (exercise_category, exercise_id) REFERENCES EXERCISE(category, id) ON DELETE CASCADE
);

CREATE INDEX WORKOUT_STEP_WORKOUT ON WORKOUT_STEP(workout);