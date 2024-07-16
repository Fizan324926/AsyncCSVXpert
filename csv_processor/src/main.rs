use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use futures::TryStreamExt;
use reqwest::Client;
use tokio::sync::Semaphore;
use dotenv::dotenv;
use std::env;
mod csv_processor;
use csv_processor::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CsvRecord {
    id: String,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressUpdate {
    success_count: u32,
    unsuccess_count: u32,
    total_records: u32,
    records_processed: u32,
    result: UrlResult,
    status_code_stats: HashMap<u16, u32>,
}

struct AppState {
    progress: Mutex<u32>,
    results: Mutex<Vec<ProgressUpdate>>,
    status_code_stats: Mutex<HashMap<u16, u32>>,
    total_records: Mutex<u32>,
    records_processed: Mutex<u32>,
}

async fn process_csv_data(
    data: web::Json<Vec<CsvRecord>>,
    app_state: web::Data<Arc<AppState>>,
    semaphore: web::Data<Arc<Semaphore>>,
) -> impl Responder {
    let records = data.into_inner();
    let mut success_count = 0;
    let mut unsuccess_count = 0;

    {
        // Increment total_records outside of the async block
        let mut total_records = app_state.total_records.lock().unwrap();
        *total_records = records.len() as u32;
    }

    // FuturesUnordered to handle multiple async tasks concurrently
    let mut futures = FuturesUnordered::new();

    for record in records {
        let app_state = app_state.clone();
        let semaphore = semaphore.clone();

        let future = async move {
            // Acquire permit from semaphore
            let _permit = semaphore.acquire().await.unwrap_or_else(|e| {
                panic!("Failed to acquire semaphore permit: {:?}", e);
            });

            // Fetch URL result
            let result = fetch_url_result(record.id.clone(), record.url.clone()).await.unwrap_or_else(|e| {
                println!("Error fetching URL result for record {:?}: {:?}", record, e);
                UrlResult {
                    id: record.id.clone(),
                    domain: record.url.clone(),
                    protocol: "".to_string(),
                    response_code: 0,
                    response_time: 0,
                    full_response: format!("Error fetching URL: {:?}", e),
                }
            });

            // Release permit explicitly (if necessary)
            drop(_permit);

            // Update success/failure counts
            if result.response_code == 200 {
                success_count += 1;
            } else {
                unsuccess_count += 1;
            }

            // Update shared state
            {
                let mut records_processed = app_state.records_processed.lock().unwrap();
                *records_processed += 1;

                let mut progress = app_state.progress.lock().unwrap();
                *progress += 1;

                let mut results = app_state.results.lock().unwrap();
                let mut status_code_stats = app_state.status_code_stats.lock().unwrap();
                *status_code_stats.entry(result.response_code).or_insert(0) += 1;

                results.push(ProgressUpdate {
                    success_count,
                    unsuccess_count,
                    total_records: *app_state.total_records.lock().unwrap(),
                    records_processed: *records_processed,
                    result: result.clone(),
                    status_code_stats: status_code_stats.clone(),
                });
            }

            // Create progress update
            let update = ProgressUpdate {
                success_count,
                unsuccess_count,
                total_records: *app_state.total_records.lock().unwrap(),
                records_processed: *app_state.records_processed.lock().unwrap(),
                result,
                status_code_stats: app_state.status_code_stats.lock().unwrap().clone(),
            };

            // Convert to Bytes and return Result<actix_web::web::Bytes, _>
            let bytes = actix_web::web::Bytes::from(serde_json::to_string(&update).unwrap());
            Ok::<actix_web::web::Bytes, actix_web::Error>(bytes)
        };

        futures.push(future);
    }

    // Stream futures concurrently
    let stream = futures.into_stream().map(|result: Result<actix_web::web::Bytes, actix_web::Error>| {
        match result {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(actix_web::error::ErrorInternalServerError("Internal Server Error")),
        }
    });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(stream)
}

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
    println!("Server started on http://127.0.0.1:8080 with {} parallel processes", max_parallel_tasks);

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
    .bind("127.0.0.1:8080")?
    .run()
    .await

    
}

