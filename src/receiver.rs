// A module that receive log events from UDP socket and insert them into a sqlite database.

use std::net::UdpSocket;
use std::str;

use crate::database::{Log, insert_log};

// Create a UDP socket listener

pub async fn udp_listener(db_path: &str, addr: &str, port: u16) -> std::io::Result<()> {
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
    let socket = UdpSocket::bind(format!("{}:{}", addr, port))?;
    println!("UDP socket listening on {}:{}", addr, port);

    // Create a buffer to store received data
    let mut buf = [0; 65535];

    // Receive log events in JSON format with all the fields and insert into database

    loop {
        // Receive data from socket
        let (amt, src) = socket.recv_from(&mut buf)?;
        // Convert the received data into a string
        let received = match str::from_utf8(&buf[..amt]) {
            Ok(v) => v,
            Err(e) => {
                println!("Error converting to string: {}", e);
                //exit program
                continue;
            }
        };
        // Print the received data
        println!("Received data from {}: {}", src, received);
        
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

