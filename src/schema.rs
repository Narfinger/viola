table! {
    tracks (id) {
        id -> Integer,
        title -> Nullable<Text>,
        artist -> Nullable<Text>,
        album -> Nullable<Text>,
        year -> Nullable<Integer>,
        path -> Text,
        duration -> Nullable<Integer>,
        albumpath -> Nullable<Text>,
    }
}
