-- Support for additional subsports

ALTER TABLE SESSION ADD COLUMN sub_sport TEXT NOT NULL DEFAULT "strength_training";
UPDATE DEVICE SET last_sync=NULL WHERE 1=1;