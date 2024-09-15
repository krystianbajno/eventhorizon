use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct City {
    pub name: String,
    pub loc: Location,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub coordinates: Vec<f64>, // Latitude, Longitude
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataEntry {
    pub filepath: String,
    pub title: String,
    pub url: String,
    pub collection_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub link: String,
    pub filepath: String,
    pub collection_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsByCity {
    pub city: String,
    pub coordinates: Option<Vec<f64>>,
    pub news: Vec<NewsItem>,
}
