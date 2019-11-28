use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use crate::playlist::restore_playlists;
use crate::types::*;

fn playlist(state: web::Data<DBPool>, req: HttpRequest) -> HttpResponse {
    let pls = restore_playlists(&*state).unwrap();
    HttpResponse::Ok().json(pls.get(0).unwrap().items)
}

pub fn run(pool: DBPool) {
    let counter = web::Data::new(pool);

    HttpServer::new(|| {
        App::new()
            .register_data(counter.clone())
            .service(fs::Files::new("/static/", "web_gui/dist/").show_files_listing())
            .service(fs::Files::new("/", "./web_gui/").index_file("index.html"))
            .service(web::resource("/playlist/").route(web::get().to(playlist)))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
