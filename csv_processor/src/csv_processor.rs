use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::time::Instant;
use url::{Url, ParseError};
use actix_web::web;
use std::fs;
use std::path::Path;
use reqwest::header::HeaderMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrlResult {
    pub id: String,
    pub domain: String,
    pub protocol: String,
    pub response_code: u16,
    pub response_time: u64,
    pub full_response: String,
}

pub async fn delete_csv_file(task_id: &str, csv_save_dir: &str) -> Result<(), String> {
    let file_name = format!("{}.csv", task_id);
    let file_path = Path::new(csv_save_dir).join(&file_name);

    if file_path.exists() {
        if let Err(err) = fs::remove_file(&file_path) {
            return Err(format!("Failed to delete CSV file: {}", err));
        }

        Ok(())
    } else {
        Err(format!("CSV file not found: {:?}", &file_path))
    }
}

fn validate_and_filter_domains(id: String, domain: String) -> Option<Vec<(String, Url, String)>> {
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
