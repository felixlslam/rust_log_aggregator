use sqlx::{Sqlite, SqlitePool, Result, Pool, FromRow};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Log {
    pub timestamp: String,
    pub app: String,
    pub host: String,
    pub filename: String,
    pub log: String,
}

// Create a function for sqlite database connection

pub async fn db_connection(path: &str) -> Result<Pool<Sqlite>> {
    let url = format!("sqlite://{}", path);
    let conn = SqlitePool::connect(&url).await?;
    Ok(conn)
}

// Create a function to initialize the database (if not exists) and create the tables (if not exists)

pub async fn create_tables(conn: &Pool<Sqlite>) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(conn)
        .await?;
    Ok(())
}

// Create a function to migrate the database

pub async fn migrate_database(conn: &Pool<Sqlite>) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(conn)
        .await?;
    Ok(())
}

// Create a function to insert a Log object into logevents table

pub async fn insert_log(
    conn:  &Pool<Sqlite>,
    log: &Log,
) -> Result<()>{
    match sqlx::query(
        "INSERT INTO logevents (timestamp, app, host, filename, log) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&log.timestamp)
    .bind(&log.app)
    .bind(&log.host)
    .bind(&log.filename)
    .bind(&log.log)
    .execute(conn)
    .await {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e.into());
        }
    }
}

// Create a function to return all the log events that are in the database

pub async fn get_logs(conn: &Pool<Sqlite>) -> Result<Vec<Log>> {
    let logs = sqlx::query_as::<_, Log>("SELECT * FROM logevents")
        .fetch_all(conn)
        .await;
    match logs {
        Ok(logs) => {
            return Ok(logs);
        }
        Err(e) => {
            println!("Error getting logs: {}", e);
            return Err(e.into());
        }
    }
}