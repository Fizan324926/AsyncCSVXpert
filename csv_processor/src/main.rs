use actix_web::{web, App, HttpServer, HttpResponse};
use actix_cors::Cors;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};

mod csv_processor;

// Shared state to simulate processing
struct AppState {
    progress: Mutex<u32>,
    results: Mutex<Vec<ProcessingResult>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProcessingResult {
    id: String,
    URL: String,
    responseCode: String,
    responseTime: String,
    fullResponse: String,
}

async fn process_csv_data(data: web::Json<Vec<HashMap<String, String>>>, app_data: web::Data<AppState>) -> HttpResponse {
    println!("Received data: {:?}", data);

    // Simulate processing each entry with a delay
    for entry in data.iter() {
        println!("Processing entry: {:?}", entry);

        // Simulate processing
        thread::sleep(Duration::from_secs(1));
        println!("Simulated processing complete for entry: {:?}", entry);

        let mut progress = app_data.progress.lock().unwrap();
        *progress += 1;
        println!("Updated progress: {}", *progress);

        let result = ProcessingResult {
            id: entry.get("id").unwrap_or(&String::from("unknown")).clone(),
            URL: entry.get("URL").unwrap_or(&String::from("unknown")).clone(),
            responseCode: String::from("200"), // Simulate a successful response
            responseTime: String::from("100ms"), // Simulate response time
            fullResponse: String::from("{}"), // Simulate a full response
        };

        let mut results = app_data.results.lock().unwrap();
        results.push(result);
        println!("Current results: {:?}", *results);
    }

    println!("All entries processed");
    HttpResponse::Ok().json("Processing completed")
}

async fn get_progress(app_data: web::Data<AppState>) -> HttpResponse {
    let progress = app_data.progress.lock().unwrap();
    let results = app_data.results.lock().unwrap();
    println!("Progress: {}", *progress);
    println!("Results: {:?}", *results);

    HttpResponse::Ok().json(serde_json::json!({
        "progress": *progress,
        "results": *results
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Shared state to track progress
    let app_state = web::Data::new(AppState {
        progress: Mutex::new(0),
        results: Mutex::new(vec![]),
    });

    // println!("Starting server with initial state: {:?}", app_state);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        println!("Setting up CORS middleware");

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .service(web::resource("/process-csv")
                .route(web::post().to(process_csv_data)))
            .service(web::resource("/progress")
                .route(web::get().to(get_progress)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
