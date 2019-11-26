use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder};

use crate::types::*;

pub fn run(pool: &DBPool) {
    HttpServer::new(|| {
        App::new().service(fs::Files::new("../web_gui/node_modules/", ".").show_files_listing())
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
