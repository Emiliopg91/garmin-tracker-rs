-- Add aditional indexes

CREATE INDEX IF NOT EXISTS SERIE_PR ON SERIE(pr);

CREATE INDEX IF NOT EXISTS SERIE_EXERCISE_PR ON SERIE(exercise_category, exercise_id, pr);