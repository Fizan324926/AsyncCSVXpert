use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::sync::atomic::AtomicUsize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvRecord {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub success_count: u32,
    pub unsuccess_count: u32,
    pub total_records: u32,
    pub records_processed: u32,
    pub result: UrlResult,
    pub status_code_stats: HashMap<u16, u32>,
}

pub struct AppState {
    pub progress: Mutex<u32>,
    pub results: Mutex<Vec<ProgressUpdate>>,
    pub status_code_stats: Mutex<HashMap<u16, Arc<AtomicUsize>>>,
    pub total_records: Mutex<u32>,
    pub records_processed: Mutex<u32>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrlResult {
    pub id: String,
    pub domain: String,
    pub protocol: String,
    pub response_code: u16,
    pub response_time: u64,
    pub full_response: String,
}
