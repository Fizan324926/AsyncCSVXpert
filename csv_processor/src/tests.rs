#![allow(warnings)]

mod app_state;
mod csv_processor;

use actix_web::http::StatusCode;
use actix_web::{web, App, HttpResponse, Responder, test};
use actix_web::body::MessageBody;

use crate::app_state::*;
use crate::csv_processor::*;

use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use std::time::Instant;
use std::fs;
use std::path::Path;



#[derive(Serialize, Deserialize, Clone)]
struct MockCsvRecord {
    id: String,
    url: String,
}

#[actix_web::test]
async fn test_validate_and_filter_domains() {
    let valid_domains = vec![
        ("1".to_string(), "google.com".to_string()),
        ("2".to_string(), "http://facebook.com".to_string()),
        ("3".to_string(), "https://youtube.com".to_string()),
    ];

    for (id, domain) in valid_domains {
        let result = validate_and_filter_domains(id.clone(), domain.clone());
        assert!(result.is_some());
        let urls = result.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].0, id);
       
    }

    let invalid_domains = vec!["user@example.com", "invalid domain"];
    for domain in invalid_domains {
        let result = validate_and_filter_domains("4".to_string(), domain.to_string());
        assert!(result.is_none());
    }
}

#[actix_web::test]
async fn test_fetch_url_result() {
    let url = "https://httpbin.org/status/200";
    let result = fetch_url_result("1".to_string(), url.to_string()).await;
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.response_code, 200);
    assert_eq!(result.domain, url);
}


#[actix_web::test]
async fn test_process_csv_data() {
    let app_state = Arc::new(AppState {
        progress: Mutex::new(0),
        results: Mutex::new(vec![]),
        status_code_stats: Mutex::new(HashMap::new()),
        total_records: Mutex::new(0),
        records_processed: Mutex::new(0),
    });

    let semaphore = Arc::new(Semaphore::new(5));

    let records = vec![
        CsvRecord {
            id: "1".to_string(),
            url: "https://httpbin.org/status/200".to_string(),
        },
        CsvRecord {
            id: "2".to_string(),
            url: "https://httpbin.org/status/404".to_string(),
        },
    ];

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(semaphore.clone()))
            .route("/process", web::post().to(process_csv_data))
    ).await;

    let req = test::TestRequest::post()
        .uri("/process")
        .set_json(&records)
        .to_request();

    let mut response = test::call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::OK);

  
  
}