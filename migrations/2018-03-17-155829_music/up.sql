-- Your SQL goes here
CREATE TABLE tracks (
    id Integer PRIMARY KEY NOT NULL,
    title VARCHAR NOT NULL,
    artist VARCHAR NOT NULL,
    album VARCHAR NOT NULL,
    genre VARCHAR NOT NULL,
    tracknumber Integer,
    year Integer,
    path VARCHAR UNIQUE NOT NULL,
    length Integer NOT NULL,
    albumpath VARCHAR
);