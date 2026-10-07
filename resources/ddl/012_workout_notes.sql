--- Notes for session and steps

ALTER TABLE workout ADD COLUMN notes TEXT CHECK (length(notes) <= 200);

ALTER TABLE workout_step ADD COLUMN notes TEXT CHECK (length(notes) <= 200);