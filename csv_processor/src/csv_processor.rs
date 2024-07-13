use actix_web::{web, HttpResponse};
use std::collections::HashMap;

pub async fn process_csv_entry(_entry: &HashMap<String, String>) -> HttpResponse {
    // Dummy processing logic (sleep for 1 second)
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    HttpResponse::Ok().json("Processed entry")
}
