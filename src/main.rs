
use std::thread;
use crate::database::{db_connection, create_tables};
use actix_web::{rt, get, HttpResponse, Responder};

pub mod database;
pub mod receiver;
const DB_PATH: &str = "logs.db";

//An actix-web service to return all the log events in the database
#[get("/logs")]
async fn get_logs() -> impl Responder {
    let conn = db_connection(DB_PATH).await.unwrap();
    let logs = match database::get_logs(&conn).await {
        Ok(v) => v,
        Err(e) => {
            println!("Error getting logs: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(logs)
}


#[actix_web::main]
async fn main() {
    println!("Creating database connection...");
    let conn = db_connection(DB_PATH).await.unwrap();
    println!("Database connection created!");
    
    println!("Creating tables...");
    create_tables(&conn).await.unwrap();
    println!("Tables created!");

    // Start the receiver loop in a separate thread

    println!("Starting UDP listener...");
    let addr = "127.0.0.1";
    const PORT: u16 = 7878;
    let handle = thread::spawn(move || receiver::udp_listener(&DB_PATH, addr, PORT));

    // Start the actix-web server at port 8080

    println!("Starting actix-web server...");
    let http_server = actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .service(get_logs)
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run();

    let http_server_handle = http_server.handle();
    rt::spawn(http_server);
    
    println!("Actix-web server started!");

    // Wait for the thread to finish
    handle.join().unwrap().await.unwrap();
    http_server_handle.stop(true).await;
    
}