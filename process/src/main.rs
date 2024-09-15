use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use memmap2::Mmap;
use std::fs::File;
use regex::Regex;
use std::io::Read;
use lazy_static::lazy_static;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct City {
    name: String,
    loc: Location,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Location {
    coordinates: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MetadataEntry {
    filepath: String,
    title: String,
    url: String,
    collection_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewsItem {
    title: String,
    link: String,
    filepath: String,
    collection_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewsByCity {
    city: String,
    coordinates: Option<Vec<f64>>,
    news: Vec<NewsItem>,
}

// Use lazy_static for regex compilation (done once)
lazy_static! {
    static ref WORD_SPLIT_REGEX: Regex = Regex::new(r"[^\w]+").unwrap();
}

// Utility function to split content into words, optimizing regex reuse
fn split_content_into_words(content: &str) -> Vec<String> {
    WORD_SPLIT_REGEX
        .split(content)
        .map(|word| word.to_lowercase())
        .filter(|word| !word.is_empty())
        .collect()
}

// Detect file type using magic bytes
fn detect_file_type(filepath: &str) -> Result<String> {
    let mut file = File::open(filepath)?;
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;

    match &buffer {
        [0x89, 0x50, 0x4E, 0x47] => Ok("png".to_string()),
        [0xFF, 0xD8, 0xFF, ..] => Ok("jpeg".to_string()),
        [0x25, 0x50, 0x44, 0x46] => Ok("pdf".to_string()),
        _ => Ok("text".to_string()),
    }
}

// Memory-mapped file reading for large files
fn mmap_file(filepath: &str) -> Result<String> {
    let file = File::open(filepath)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = std::str::from_utf8(&mmap).unwrap_or("").to_string();
    Ok(content)
}

// Optimized function to check if city is near any keyword within a sentence
fn is_city_near_keywords(
    sentence_words: &[String],
    city_name: &str,
    keywords: &HashSet<String>,
) -> bool {
    let city_found = sentence_words.iter().any(|word| word == city_name);
    let keyword_found = sentence_words.iter().any(|word| keywords.contains(word));

    city_found && keyword_found
}

fn process_entry(
    entry: &MetadataEntry,
    keywords: &HashSet<String>,
    city_map: &HashMap<String, City>,
    news_by_city: Arc<Mutex<HashMap<String, Vec<NewsItem>>>>,
) -> Result<()> {
    // Skip files that are not .txt or .html
    if !(entry.filepath.ends_with(".html") || entry.filepath.ends_with(".txt")) {
        println!("Skipping file due to unsupported extension: {}", entry.filepath);
        return Ok(());
    }

    // Check file type using magic number detection
    let file_type = detect_file_type(&entry.filepath)?;
    if file_type != "text" {
        println!("Skipping non-text file: {} (detected as {})", entry.filepath, file_type);
        return Ok(());
    }

    // Memory-mapped file reading for large content
    let content = mmap_file(&entry.filepath)?;
    let sentences: Vec<&str> = content.split('.').collect(); // Split content into sentences

    // Split title into words
    let title_words: HashSet<String> = split_content_into_words(&entry.title).into_iter().collect();
    let mut relevant_cities = vec![];

    // Check if any city is in the title and directly add the news to those cities
    for city_name in city_map.keys() {
        if title_words.contains(city_name) {
            println!("City '{}' found in title, directly adding to city.", city_name);
            relevant_cities.push(city_name.clone());
        }
    }

    // Check if any keyword is in the title and mark it as relevant
    let keyword_found_in_title = title_words.iter().any(|word| keywords.contains(word));

    // If a keyword is found in the title, check content for cities in the same sentence
    if keyword_found_in_title {
        for sentence in &sentences {
            let sentence_words: Vec<String> = split_content_into_words(sentence);
            for city_name in city_map.keys() {
                if is_city_near_keywords(&sentence_words, city_name, &keywords) {
                    relevant_cities.push(city_name.clone());
                }
            }
        }
    } else {
        // If no keyword is in the title, perform strict sentence-based matching for cities
        for sentence in &sentences {
            let sentence_words: Vec<String> = split_content_into_words(sentence);
            if keywords.iter().any(|keyword| sentence_words.contains(keyword)) {
                for city_name in city_map.keys() {
                    if sentence_words.contains(city_name) {
                        relevant_cities.push(city_name.clone());
                    }
                }
            }
        }
    }

    // If no cities were found in title or content, add to UNSPECIFIED_LOCATION
    if relevant_cities.is_empty() {
        relevant_cities.push("UNSPECIFIED_LOCATION".to_string());
    }

    // Prepare news items for relevant cities
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

    // Batch update to reduce mutex contention
    let mut news_by_city_lock = news_by_city.lock().unwrap();
    for (city, news_item) in news_updates {
        news_by_city_lock.entry(city).or_insert_with(Vec::new).push(news_item);
    }

    Ok(())
}

// Running Parallel Processing for All Entries
fn run_in_parallel(
    metadata: Vec<MetadataEntry>,
    keywords: HashSet<String>,
    city_map: HashMap<String, City>,
) -> Result<HashMap<String, Vec<NewsItem>>> {
    let news_by_city: Arc<Mutex<HashMap<String, Vec<NewsItem>>>> = Arc::new(Mutex::new(HashMap::new()));

    metadata.par_iter().for_each(|entry| {
        let result = process_entry(
            entry,
            &keywords,
            &city_map,
            Arc::clone(&news_by_city),
        );
        if let Err(err) = result {
            eprintln!("Error processing entry {}: {}", entry.filepath, err);
        }
    });

    let final_result = Arc::try_unwrap(news_by_city).unwrap().into_inner().unwrap();
    Ok(final_result)
}

// Main function
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run <keyword1> <keyword2> ...");
        std::process::exit(1);
    }

    let keywords: HashSet<String> = args[1..].iter().map(|k| k.to_lowercase()).collect();
    let city_data: Vec<City> = serde_json::from_str(&fs::read_to_string("data/cities/cities-poland.json")?)?;

    // Create a HashMap for cities for O(1) lookup
    let city_map: HashMap<String, City> = city_data.into_iter()
        .map(|city| (city.name.to_lowercase(), city))
        .collect();

    let metadata: Vec<MetadataEntry> = serde_json::from_str(&fs::read_to_string("data/output/metadata.json")?)?;

    let news_by_city = run_in_parallel(metadata, keywords, city_map)?;

    let output_json = serde_json::to_string_pretty(&news_by_city)?;
    fs::create_dir_all("data/mapped")?;
    fs::write("data/mapped/news_by_city.json", output_json)?;

    println!("Processing completed. Results saved to 'data/mapped/news_by_city.json'.");
    Ok(())
}
