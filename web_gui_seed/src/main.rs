use seed::{prelude::*, *};
use serde;
#[macro_use]
use serde_json;
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Track {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub tracknumber: Option<i32>,
    pub year: Option<i32>,
    pub path: String,
    pub length: i32,
    pub albumpath: Option<String>,
    pub playcount: Option<i32>,
}

struct Model {
    tracks: Vec<Track>,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::Init);
    Model { tracks: vec![] }
}
enum Msg {
    Init,
    InitRecv(Vec<Track>),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Init => {
            orders.perform_cmd(async {
                let response = fetch("/playlist/").await.expect("HTTP request failed");
                let tracks = response
                    .check_status() // ensure we've got 2xx status
                    .expect("status check failed")
                    .json::<Vec<Track>>()
                    .await
                    .expect("deserialization failed");
                Msg::InitRecv(tracks)
            });
        }
        Msg::InitRecv(t) => {
            model.tracks = t;
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![table![
        C!["table"],
        model
            .tracks
            .iter()
            .map(|t| { tr![td![&t.title], td![&t.artist], td![&t.album]] })
    ]]
}

fn main() {
    App::start("app", init, update, view);
    println!("Hello, world!");
}
