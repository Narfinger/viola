table! {
    playlists (id) {
        id -> Integer,
        name -> Text,
        current_position -> Integer,
    }
}

table! {
    playlisttracks (id) {
        id -> Integer,
        playlist_id -> Integer,
        track_id -> Integer,
        playlist_order -> Integer,
    }
}

table! {
    tracks (id) {
        id -> Integer,
        title -> Text,
        artist -> Text,
        album -> Text,
        genre -> Text,
        tracknumber -> Nullable<Integer>,
        year -> Nullable<Integer>,
        path -> Text,
        length -> Integer,
        albumpath -> Nullable<Text>,
    }
}

joinable!(playlisttracks -> playlists (playlist_id));
joinable!(playlisttracks -> tracks (track_id));

allow_tables_to_appear_in_same_query!(
    playlists,
    playlisttracks,
    tracks,
);
