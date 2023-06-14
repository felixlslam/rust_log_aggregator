// A module that receive log events from UDP socket and insert them into a sqlite database.

use tokio::net::UdpSocket;
use std::io;
use std::str;
use crate::database::{Log, insert_log};

// Create a UDP socket listener

pub async fn udp_listener(db_path: &str, addr: &str, port: u16) -> io::Result<()> {
    // Create a database connection
    let conn = match sqlx::sqlite::SqlitePool::connect(db_path).await {
        Ok(v) => v,
        Err(e) => {
            println!("Error connecting to database: {}", e);
            //exit program
            return Ok(());
        }
    };
    // Create a UDP socket
    let socket = UdpSocket::bind(format!("{}:{}", addr, port)).await?;

    println!("UDP socket listening on {}:{}", addr, port);

    // Create a buffer to store received data
    let mut buf = Vec::with_capacity(1024);

    // Receive log events in JSON format with all the fields and insert into database
    loop {
        // Receive data from socket
        socket.readable().await?;
        buf.clear();
        match socket.try_recv_buf(&mut buf) {
            Ok(n) => {
                println!("GOT {:?}", &buf[..n]);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        }

        // Convert the received data into a string
        let received = match str::from_utf8(&buf) {
            Ok(v) => v,
            Err(e) => {
                println!("Error converting to string: {}", e);
                //exit program
                continue;
            }
        };
        // Print the received data
        println!("Received data {}", received);
        
        // Parse the received data into a Log struct
        let log: Log = match serde_json::from_str(received) {
            Ok(v) => v,
            Err(e) => {
                println!("Error parsing data: {}", e);
                //exit program
                continue    
            }
        };

        // Insert the log into the database
        match insert_log(&conn, &log).await {
            Ok(_) => println!("Data inserted!"),
            Err(e) => {
                println!("Error inserting data: {}", e);
                //exit program
                continue;
            }
        }
    }
}

