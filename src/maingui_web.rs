use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use crate::playlist::restore_playlists;
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
}

pub fn run(pool: DBPool) {
    let state = WebGui { pool: pool };

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
