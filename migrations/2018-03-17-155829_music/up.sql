-- Your SQL goes here
CREATE TABLE tracks (
    id Integer PRIMARY KEY NOT NULL,
    title VARCHAR,
    artist VARCHAR,
    album VARCHAR,
    genre VARCHAR,
    year Integer,
    tracknumber Integer,
    path VARCHAR NOT NULL,
    length Integer NOT NULL,
    albumpath VARCHAR
);