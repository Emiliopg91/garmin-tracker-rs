--- Move heart rate data to session table
ALTER TABLE session ADD COLUMN heart_rates BLOB;

UPDATE session
SET heart_rates = (
    SELECT unhex(GROUP_CONCAT(printf('%02x', hr), '' ORDER BY idx))
    FROM heart_rate
    WHERE heart_rate.session = session.date
)
WHERE date IN (SELECT DISTINCT session FROM heart_rate);

DROP TABLE heart_rate;