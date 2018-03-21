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
