use crate::models::{MetadataEntry, City, NewsItem, NewsByCity};
use crate::text_processing::{split_content_into_words, fuzzy_match_city};
use crate::file_io::mmap_file;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use anyhow::Result;

pub fn process_entry(
    entry: &MetadataEntry,
    keywords: &HashSet<String>,
    city_map: &HashMap<String, City>,
    news_by_city: Arc<Mutex<HashMap<String, Vec<NewsItem>>>>,
    fuzzy_threshold_title: f64,
    fuzzy_threshold_content: f64,
    proximity_threshold: usize,
    parse_content: bool
) -> Result<()> {
    let mut relevant_cities = vec![];
    let mut keyword_found = false;

    println!("Processing file: {}", entry.filepath);

    // Read the content of the file
    let content = mmap_file(&entry.filepath)?;

    // Split the title into words
    let title_words: HashSet<String> = split_content_into_words(&entry.title).into_iter().collect();

    // Check if any city matches in the title (with fuzzy matching)
    for (city_name, _) in city_map.iter() {
        if title_words.iter().any(|word| fuzzy_match_city(city_name, word, fuzzy_threshold_title)) {
            println!("City '{}' (fuzzy) found in title, adding directly to city.", city_name);
            relevant_cities.push(city_name.clone());
        }
    }

    // Check if any keyword is found in the title
    if title_words.iter().any(|word| keywords.contains(word)) {
        keyword_found = true;
        println!("Keyword found in title.");
        if relevant_cities.is_empty() {
            relevant_cities.push("UNSPECIFIED_LOCATION".to_string());
        }
    }

    // Parse content if enabled
    if parse_content && !keyword_found {
        let sentences: Vec<&str> = content.split('.').collect();
        for sentence in sentences {
            let sentence_words: HashSet<String> = split_content_into_words(sentence).into_iter().collect();
            if keywords.iter().any(|keyword| sentence_words.contains(keyword)) {
                keyword_found = true;
                for (city_name, _) in city_map.iter() {
                    if sentence_words.iter().any(|word| fuzzy_match_city(city_name, word, fuzzy_threshold_content)) {
                        relevant_cities.push(city_name.clone());
                    }
                }
            }
        }
    }

    if !keyword_found {
        return Ok(());
    }

    let mut news_updates = vec![];
    for city_name in relevant_cities {
        news_updates.push((
            city_name.clone(),
            NewsItem {
                title: entry.title.clone(),
                link: entry.url.clone(),
                filepath: entry.filepath.clone(),
                collection_date: entry.collection_date.clone(),
            },
        ));
    }

    let mut news_by_city_lock = news_by_city.lock().unwrap();
    for (city, news_item) in news_updates {
        news_by_city_lock.entry(city).or_insert_with(Vec::new).push(news_item);
    }

    Ok(())
}
