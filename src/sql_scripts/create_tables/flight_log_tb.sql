CREATE TABLE "FlightLogTb" (
                               "FlightID"	INTEGER,
                               "VelocityX"	REAL,
                               "VelocityY"	REAL,
                               "VelocityZ"	REAL,
                               "Pitch"	REAL,
                               "Yaw"	REAL,
                               "GestureID"	INTEGER,
                               "EntryTime"	INTEGER,
                               PRIMARY KEY("FlightID" AUTOINCREMENT),
                               CONSTRAINT "FlightID" FOREIGN KEY("FlightID") REFERENCES "FlightModelTb"("FlightID"),
                               CONSTRAINT "GestureID" FOREIGN KEY("GestureID") REFERENCES ""
)