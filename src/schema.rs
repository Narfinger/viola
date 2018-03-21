table! {
    tracks (id) {
        id -> Integer,
        title -> Nullable<Text>,
        artist -> Nullable<Text>,
        album -> Nullable<Text>,
        genre -> Nullable<Text>,
        tracknumber -> Nullable<Integer>,
        year -> Nullable<Integer>,
        path -> Text,
        length -> Integer,
        albumpath -> Nullable<Text>,
    }
}
