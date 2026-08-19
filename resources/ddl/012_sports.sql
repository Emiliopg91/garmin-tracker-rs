--- Sports

ALTER TABLE SESSION ADD COLUMN sport TEXT DEFAULT "Generic";
UPDATE SESSION SET sport="Strength" WHERE sub_sport="strength_training";
ALTER TABLE SESSION DROP COLUMN sub_sport;