-- Your SQL goes here
CREATE TABLE tracks (
    id Integer PRIMARY KEY NOT NULL,
    title VARCHAR,
    artist VARCHAR,
    album VARCHAR,
    year Integer,
    path VARCHAR NOT NULL,
    duration Integer,
    albumpath VARCHAR
);