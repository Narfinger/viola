table! {
    tracks (id) {
        id -> Integer,
        title -> Nullable<Text>,
        artist -> Nullable<Text>,
        album -> Nullable<Text>,
        year -> Nullable<Integer>,
        path -> Nullable<Text>,
        duration -> Nullable<Integer>,
    }
}
