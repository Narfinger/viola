-- Your SQL goes here
CREATE TABLE tracks (
    id Integer PRIMARY KEY NOT NULL,
    title VARCHAR,
    artist VARCHAR,
    album VARCHAR,
    genre VARCHAR,
    tracknumber Integer,
    year Integer,
    path VARCHAR UNIQUE NOT NULL,
    length Integer NOT NULL,
    albumpath VARCHAR
);