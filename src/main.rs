
use crate::database::{db_connection, create_tables};
use std::convert::Infallible;
use std::net::SocketAddr;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use tokio::net::TcpListener;
use std::io;

pub mod database;
pub mod receiver;
pub mod web;
const LOCAL_ADDRESS: &'static str = "127.0.0.1";
const DB_PATH: &str = "logs.db";
const UDP_PORT: u16 = 7878;
const HTTP_PORT: u16 = 8080;
/*
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
 */

// Create a hello service function

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::from("Hello World!")))
}

// Create a logs service that returns log events in JSON format

async fn logs(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let conn = db_connection(DB_PATH).await.unwrap();
    let logs = match database::get_logs(&conn).await {
        Ok(v) => v,
        Err(e) => {
            println!("Error getting logs: {}", e);
            return Ok(Response::new(Full::from("Error getting logs")));
        }
    };
    let logs_json = match serde_json::to_string(&logs) {
        Ok(v) => v,
        Err(e) => {
            println!("Error converting logs to JSON: {}", e);
            return Ok(Response::new(Full::from("Error converting logs to JSON")));
        }
    };
    Ok(Response::new(Full::from(logs_json)))
}

// Create a response for HTTP 404 error

fn not_found() -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(404)
        .body(Full::from("Not Found"))
        .unwrap())
}

// Create a router service  that routes requests to the appropriate service

async fn router(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    if req.uri().path() == "/hello" {
        return hello(req).await;
    } else if req.uri().path() == "/logs" {
        return logs(req).await;
    } else {
        return Ok(not_found().unwrap())
    }
}



#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Creating database connection...");
    let conn = db_connection(DB_PATH).await.unwrap();
    println!("Database connection created!");
    
    println!("Creating tables...");
    create_tables(&conn).await.unwrap();
    println!("Tables created!");

    let addr = SocketAddr::from(([127, 0, 0, 1], HTTP_PORT));

    let listener = TcpListener::bind(addr).await?;

    tokio::select! {
        _ = async {
                loop {
                    println!("Waiting for an incoming connection...");
                    let (stream, _) = listener.accept().await?;
                    println!("Accepted a connection!");
            
                    // Spawn a tokio task to serve multiple connections concurrently
                    tokio::spawn(async move {
                        // Finally, we bind the incoming connection to our `hello` service
                        if let Err(err) = http1::Builder::new()
                            // `service_fn` converts our function in a `Service`
                            .serve_connection(stream, service_fn(router))
                            .await
                        {
                            println!("Error serving connection: {:?}", err);
                        }
                    });
                }
                #[allow(unreachable_code)]
                Ok::<_, io::Error>(())
            }  => {},
        _ = receiver::udp_listener(&DB_PATH, LOCAL_ADDRESS, UDP_PORT) => {}
    }

    Ok(())
}