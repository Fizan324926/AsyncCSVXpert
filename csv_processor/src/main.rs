#![allow(warnings)]

use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::TryStreamExt;
use reqwest::Client;
use tokio::sync::Semaphore;

mod app_state;
mod csv_processor;

use app_state::{AppState, CsvRecord, ProgressUpdate};
use csv_processor::*;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    env_logger::init();

    // Read the number of parallel processes from the environment or use a default
    let max_parallel_tasks: usize = env::var("MAX_PARALLEL_TASKS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20); // Default to 20 if not specified in .env

    // Read host and port from environment variables or use defaults
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Shared state to track progress
    let app_state = web::Data::new(Arc::new(AppState {
        progress: Mutex::new(0),
        results: Mutex::new(vec![]),
        status_code_stats: Mutex::new(HashMap::new()),
        total_records: Mutex::new(0),
        records_processed: Mutex::new(0),
    }));

    // Semaphore to limit concurrent tasks
    let semaphore = web::Data::new(Arc::new(Semaphore::new(max_parallel_tasks)));
    println!(
        "Server started on http://{}:{} with {} parallel processes",
        host, port, max_parallel_tasks
    );

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .app_data(semaphore.clone())
            .wrap(cors)
            .service(web::resource("/process")
                .route(web::post().to(process_csv_data)))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
