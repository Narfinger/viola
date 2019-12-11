use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::sync::Arc;

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::playlist::restore_playlists;
use crate::playlist_tabs;
use crate::types::*;

async fn playlist(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    let pls = restore_playlists(&state.pool).unwrap();
    HttpResponse::Ok().json(
        &pls.get(0)
            .unwrap()
            .items
            .as_slice()
            .chunks(30)
            .next()
            .unwrap(),
    )
}

fn play(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    state
        .gstreamer
        .do_gstreamer_action(&gstreamer_wrapper::GStreamerAction::Playing);
    HttpResponse::Ok().finish()
}

fn pause(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    state
        .gstreamer
        .do_gstreamer_action(&gstreamer_wrapper::GStreamerAction::Pausing);
    HttpResponse::Ok().finish()
}

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<gstreamer_wrapper::GStreamer>,
}

pub fn run(pool: DBPool) {
    println!("Loading playlist");
    let lp = Arc::new(
        restore_playlists(&pool)
            .expect("Error restoring playlisttabs")
            .swap_remove(0),
    );

    println!("Starting gstreamer");
    let (gst, recv) =
        gstreamer_wrapper::new(lp.clone(), pool.clone()).expect("Error Initializing gstreamer");
    println!("Setting up gui");
    let state = WebGui {
        pool: pool.clone(),
        gstreamer: gst,
    };

    println!("Doing data");
    let data = web::Data::new(state);

    println!("Starting web gui on 127.0.0.1:8088");
    HttpServer::new(move || {
        App::new()
            .register_data(data.clone())
            .service(web::resource("/playlist/").route(web::get().to(playlist)))
            .service(web::resource("/transport/play").route(web::get().to(play)))
            .service(web::resource("/transport/pause").route(web::get().to(pause)))
            .service(fs::Files::new("/static/", "web_gui/dist/").show_files_listing())
            .service(fs::Files::new("/", "./web_gui/").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
