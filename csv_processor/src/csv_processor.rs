#![allow(warnings)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}};
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures::stream::{FuturesUnordered, StreamExt};
use futures::TryStreamExt;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use url::{ParseError, Url};

use crate::app_state::*;



pub fn validate_and_filter_domains(id: String, domain: String) -> Option<Vec<(String, Url, String)>> {
    let mut validated_urls = Vec::new();

    if domain.contains('@') {
        return None;
    }

    let mut url_str = domain.clone();
    let protocol = "http";
    if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
        url_str = format!("{}://{}", protocol, domain);
    }

    match Url::parse(&url_str) {
        Ok(url) => {
            validated_urls.push((id.clone(), url.clone(), protocol.to_string()));
        },
        Err(_) => {
            return None;
        },
    }

    if validated_urls.is_empty() {
        None
    } else {
        Some(validated_urls)
    }
}



pub async fn fetch_url_result(id: String, domain: String) -> Result<UrlResult, reqwest::Error> {
    if let Some(validated_urls) = validate_and_filter_domains(id.clone(), domain.clone()) {
        for (id, url, protocol) in validated_urls {
            let client = Client::new();
            let start_time = Instant::now();

            let response = client.head(url.clone()).send().await;

            let response_time = start_time.elapsed().as_millis() as u64;
            let response_code = response.as_ref().map_or(0, |res| res.status().as_u16());
            let headers = response.as_ref().map_or_else(
                |_| HeaderMap::new(), // Accepts and ignores the argument (None case)
                |res| res.headers().clone(), // Takes the response and retrieves headers
            );

            let result = UrlResult {
                id: id.clone(),
                domain: domain.clone(),
                protocol: protocol.clone(),
                response_code,
                response_time,
                full_response: match response {
                    Ok(_) => format!("Headers: {:?}", headers),
                    Err(_) => "Error making request".to_string(),
                },
            };

            return Ok(result);
        }
    }

    let result = UrlResult {
        id,
        domain,
        protocol: "".to_string(),
        response_code: 0,
        response_time: 0,
        full_response: "INVALID domain or URL".to_string(),
    };

    Ok(result)
}

pub async fn process_csv_data(
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

                // Update the atomic counter for the status code
                let mut status_code_stats = app_state.status_code_stats.lock().unwrap();
                if let Some(counter) = status_code_stats.get(&result.response_code) {
                    counter.fetch_add(1, Ordering::SeqCst);
                } else {
                    status_code_stats.insert(result.response_code, Arc::new(AtomicUsize::new(1)));
                }

                let status_code_stats_snapshot: HashMap<u16, u32> = status_code_stats.iter()
                    .map(|(&code, counter)| (code, counter.load(Ordering::SeqCst) as u32))
                    .collect();

                results.push(ProgressUpdate {
                    success_count,
                    unsuccess_count,
                    total_records: *app_state.total_records.lock().unwrap(),
                    records_processed: *records_processed,
                    result: result.clone(),
                    status_code_stats: status_code_stats_snapshot,
                });
            }

            // Create progress update
            let update = ProgressUpdate {
                success_count,
                unsuccess_count,
                total_records: *app_state.total_records.lock().unwrap(),
                records_processed: *app_state.records_processed.lock().unwrap(),
                result,
                status_code_stats: app_state.status_code_stats.lock().unwrap().iter()
                    .map(|(&code, counter)| (code, counter.load(Ordering::SeqCst) as u32))
                    .collect(),
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
