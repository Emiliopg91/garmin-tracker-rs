--- Table for pending workout upload to device

CREATE TABLE WORKOUT_DEVICE (
    workout   TEXT NOT NULL,
    device    TEXT NOT NULL,
    
    PRIMARY KEY(workout, device),
    FOREIGN KEY(workout) REFERENCES WORKOUT(name),
    FOREIGN KEY(device) REFERENCES DEVICE(serial)
);

CREATE INDEX WORKOUT_DEVICE_WORKOUT ON WORKOUT_DEVICE(workout);
CREATE INDEX WORKOUT_DEVICE_DEVICE ON WORKOUT_DEVICE(device);


