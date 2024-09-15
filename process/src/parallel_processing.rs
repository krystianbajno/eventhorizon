use crate::models::{City, MetadataEntry, NewsByCity, NewsItem};
use crate::processing::process_entry;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub fn run_in_parallel(
    metadata: Vec<MetadataEntry>,
    keywords: HashSet<String>,
    city_map: HashMap<String, City>,
    fuzzy_threshold_title: f64,
    fuzzy_threshold_content: f64,
    proximity_threshold: usize,
    parse_content: bool
) -> Result<Vec<NewsByCity>> {
    let news_by_city: Arc<Mutex<HashMap<String, Vec<NewsItem>>>> = Arc::new(Mutex::new(HashMap::new()));

    metadata.par_iter().for_each(|entry| {
        let result = process_entry(
            entry,
            &keywords,
            &city_map,
            Arc::clone(&news_by_city),
            fuzzy_threshold_title,
            fuzzy_threshold_content,
            proximity_threshold,
            parse_content
        );
        if let Err(err) = result {
            eprintln!("Error processing entry {}: {}", entry.filepath, err);
        }
    });

    let news_by_city_map = Arc::try_unwrap(news_by_city).unwrap().into_inner().unwrap();
    let news_by_city_vec: Vec<NewsByCity> = news_by_city_map.into_iter().map(|(city_name, news)| {
        let coordinates = city_map.get(&city_name.to_lowercase()).map(|city| city.loc.coordinates.clone());
        NewsByCity {
            city: city_name,
            coordinates,
            news,
        }
    }).collect();

    Ok(news_by_city_vec)
}