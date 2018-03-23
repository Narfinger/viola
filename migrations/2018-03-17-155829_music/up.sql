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

CREATE TABLE playlists (
    id Integer PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    current_position Integer NOT NULL
);

CREATE TABLE playlisttracks (
    id Integer PRIMARY KEY NOT NULL,
    playlist_id Integer NOT NULL,
    track_id Integer NOT NULL,
    playlist_order Integer NOT NULL,
    UNIQUE(playlist_id, playlist_order),
    FOREIGN KEY(playlist_id) REFERENCES playlist(id),
    FOREIGN KEY(track_id) REFERENCES tracks(id)
);