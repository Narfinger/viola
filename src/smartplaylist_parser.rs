use toml;

fn read_file() -> Query {
    let mut file = File::open("tests/playlists.toml");
    let s = Vec::new();
    toml::from_str::<Vec<HashMap<String,String>>>(&s).expect("Could not parse");

    s.into_iter().map(|table| {
        table.map(|key, value| {
            match key {
                "name" => 
                "artist_include" =>
                "dir_include" =>
                "dir_exclude" =>
                "genre_include" =>
                v => println!("We found a weird tag, we could not quite figure out: {}", v);
            }
        })
    });
}