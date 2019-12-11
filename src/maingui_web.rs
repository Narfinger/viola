use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::sync::Arc;

use crate::gstreamer_wrapper;
use crate::playlist::restore_playlists;
use crate::playlist_tabs;
use crate::types::*;

fn playlist(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
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

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<gstreamer_wrapper::GStreamer>,
}

pub fn run(pool: DBPool) {
    let lp = Arc::new(
        restore_playlists(&pool)
            .expect("Error restoring playlisttabs")
            .swap_remove(0),
    );

    let (gst, recv) =
        gstreamer_wrapper::new(lp.clone(), pool).expect("Error Initializing gstreamer");
    let state = WebGui {
        pool: pool,
        gstreamer: gst,
    };

    let data = web::Data::new(state);

    println!("Starting web gui on 127.0.0.1:8088");
    HttpServer::new(move || {
        App::new()
            .register_data(data.clone())
            .service(web::resource("/playlist/").route(web::get().to(playlist)))
            .service(fs::Files::new("/static/", "web_gui/dist/").show_files_listing())
            .service(fs::Files::new("/", "./web_gui/").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
