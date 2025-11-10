CREATE TABLE "FlightLogTb" (
    "FlightID"	INTEGER,
    "VelocityX"	REAL,
    "VelocityY"	REAL,
    "VelocityZ"	REAL,
    "Pitch"	REAL,
    "Yaw"	REAL,
    "GestureID"	INTEGER,
    "EntryTime"	INTEGER,
    CONSTRAINT "FlightID" FOREIGN KEY("FlightID") REFERENCES "FlightModelTb"("FlightID"),
    CONSTRAINT "GestureID" FOREIGN KEY("GestureID") REFERENCES "GestureControlTb"
)