use std::thread;

use druid::widget::{prelude::*, List, Scroll};
use druid::widget::{Flex, Label, TextBox};
use druid::{AppLauncher, Color, Data, Lens, UnitPoint, WidgetExt, WindowDesc};
use im::Vector;
use reqwest::Url;

#[derive(Clone, Data, Lens)]
struct TrackStub {
    title: String,
    artist: String,
}

#[derive(Clone, Data, Lens)]
struct AppData {
    tracks: Vector<TrackStub>,
}

fn update_state(event_sink: druid::ExtEventSink) {
    let tracks: Vec<viola_common::Track> =
        reqwest::blocking::get("http://127.0.0.1:8080/playlist/1/")
            .unwrap()
            .json()
            .unwrap();
    let modified_tracks: Vector<TrackStub> = tracks
        .iter()
        .map(|t| TrackStub {
            title: t.title.clone(),
            artist: t.artist.clone(),
        })
        .take(1000)
        .collect();
    event_sink.add_idle_callback(move |data: &mut AppData| {
        data.tracks = modified_tracks;
    });
}

pub fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title("Hello World!")
        .window_size((400.0, 400.0));

    // create the initial app state

    let tracks = (1..1000)
        .map(|i| TrackStub {
            title: format!("title {}", i).to_string(),
            artist: format!("artist {}", i).to_string(),
        })
        .collect();

    let initial_state = AppData { tracks };

    // start the application. Here we pass in the application state.
    let launcher = AppLauncher::with_window(main_window).log_to_console();

    let event_sink = launcher.get_external_handle();
    thread::spawn(move || update_state(event_sink));
    launcher
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<AppData> {
    let list = Scroll::new(List::new(|| {
        Flex::row()
            .with_child(
                Label::new(|item: &TrackStub, _env: &_| format!("{}", item.artist))
                    .align_vertical(UnitPoint::LEFT)
                    .padding(10.0)
                    .fix_width(100.0)
                    .fix_height(50.0)
                    .background(Color::rgb(0.5, 0.5, 0.5)),
            )
            .with_child(
                Label::new(|item: &TrackStub, _env: &_| format!("{}", item.title))
                    .align_vertical(UnitPoint::LEFT)
                    .padding(10.0)
                    .fix_height(50.0)
                    .fix_width(100.0)
                    .background(Color::rgb(0.5, 0.5, 0.5)),
            )
    }))
    .vertical()
    .fix_height(500.0)
    .lens(AppData::tracks);

    // arrange the two widgets vertically, with some padding
    Flex::column()
        .with_child(list)
        .align_vertical(UnitPoint::CENTER)
}
