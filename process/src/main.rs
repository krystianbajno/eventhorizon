use std::fs;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use memmap2::Mmap;
use std::fs::File;
use regex::Regex;
use lazy_static::lazy_static;
use scraper::{Html, Selector}; // HTML parser
use strsim::jaro_winkler; // For fuzzy city matching

#[derive(Debug, Serialize, Deserialize, Clone)]
struct City {
    name: String,
    loc: Location,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Location {
    coordinates: Vec<f64>, // Latitude, Longitude
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

lazy_static! {
    static ref WORD_SPLIT_REGEX: Regex = Regex::new(r"[^\w]+").unwrap();
}

// Utility function to split content into words (lowercased, alphanumeric only)
fn split_content_into_words(content: &str) -> Vec<String> {
    WORD_SPLIT_REGEX
        .split(content)
        .map(|word| word.to_lowercase())
        .filter(|word| !word.is_empty())
        .collect()
}

// Memory-mapped file reading for large files
fn mmap_file(filepath: &str) -> Result<String> {
    let file = File::open(filepath)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = std::str::from_utf8(&mmap).unwrap_or("").to_string();
    Ok(content)
}

// Function to check if a city name matches a word with fuzzy logic
fn fuzzy_match_city(city_name: &str, word: &str, threshold: f64) -> bool {
    jaro_winkler(city_name, word) >= threshold
}

// Function to check if a keyword or city match is inside an <a> tag in content
fn is_inside_link(html: &Html, matched_text: &str) -> bool {
    let a_selector = Selector::parse("a").unwrap();
    for element in html.select(&a_selector) {
        for text in element.text() {
            if text.contains(matched_text) {
                return true; // The matched text is inside an <a> tag
            }
        }
    }
    false
}

// Function to check if a city name and keyword appear within close proximity in the content
fn city_and_keyword_in_proximity(
    sentence_words: &HashSet<String>,
    city_name: &str,
    keywords: &HashSet<String>,
    proximity_threshold: usize
) -> bool {
    let sentence_word_list: Vec<&String> = sentence_words.iter().collect();

    // Find positions of the city and keywords in the sentence
    let city_positions: Vec<usize> = sentence_word_list.iter()
        .enumerate()
        .filter(|(_, word)| **word == city_name)
        .map(|(i, _)| i)
        .collect();

    let keyword_positions: Vec<usize> = sentence_word_list.iter()
        .enumerate()
        .filter(|(_, word)| keywords.contains(**word))
        .map(|(i, _)| i)
        .collect();

    // Check if any city and keyword are within proximity
    for city_pos in city_positions {
        for keyword_pos in &keyword_positions {
            if city_pos.abs_diff(*keyword_pos) <= proximity_threshold {
                return true;
            }
        }
    }

    false
}

// Function to process a single metadata entry, matching cities and keywords, and updating the news_by_city map
fn process_entry(
    entry: &MetadataEntry,
    keywords: &HashSet<String>,
    city_map: &HashMap<String, City>,
    news_by_city: Arc<Mutex<HashMap<String, Vec<NewsItem>>>>,
    fuzzy_threshold_title: f64,    // Fuzzy matching threshold for title
    fuzzy_threshold_content: f64,  // Strict matching threshold for content
    proximity_threshold: usize,    // Proximity threshold for matching in content
    relaxed: bool                  // Allow matching inside links if true
) -> Result<()> {
    let mut relevant_cities = vec![];
    let mut keyword_found = false;

    println!("Processing file: {}", entry.filepath);

    // Read the content of the file and parse HTML
    let content = mmap_file(&entry.filepath)?;
    let html = Html::parse_document(&content);

    // Split the title into words
    let title_words: HashSet<String> = split_content_into_words(&entry.title).into_iter().collect();
    println!("Title words: {:?}", title_words);

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
        // If no city is found in the title but a keyword is, assign to UNSPECIFIED_LOCATION
        if relevant_cities.is_empty() {
            println!("Keyword found in title, but no city, assigning to UNSPECIFIED_LOCATION.");
            relevant_cities.push("UNSPECIFIED_LOCATION".to_string());
        }
    }

    // If no keyword is found in the title, fallback to content-based matching
    if !keyword_found {
        let sentences: Vec<&str> = content.split('.').collect();
        println!("Processing content in {} sentences.", sentences.len());

        // Check for city and keyword in the same sentence (strict fuzzy matching and proximity check)
        for (i, sentence) in sentences.iter().enumerate() {
            let sentence_words: HashSet<String> = split_content_into_words(sentence).into_iter().collect();

            // Check if a keyword is found in the sentence
            if keywords.iter().any(|keyword| sentence_words.contains(keyword)) {
                keyword_found = true;
                println!("Keyword found in sentence {}.", i + 1);

                // If the keyword is inside an <a> tag, skip unless relaxed mode is on
                if !relaxed && is_inside_link(&html, sentence) {
                    println!("Keyword match inside <a> tag in sentence {}, skipping.", i + 1);
                    continue;
                }

                // If a city is found in the same sentence, and it is within close proximity to the keyword
                for (city_name, _) in city_map.iter() {
                    if sentence_words.iter().any(|word| fuzzy_match_city(city_name, word, fuzzy_threshold_content)) {
                        // Check if the city match is inside an <a> tag
                        if !relaxed && is_inside_link(&html, city_name) {
                            println!("City '{}' match inside <a> tag in content, skipping.", city_name);
                            continue;
                        }

                        if city_and_keyword_in_proximity(&sentence_words, city_name, keywords, proximity_threshold) {
                            println!("City '{}' (fuzzy) found in sentence {} near a keyword.", city_name, i + 1);
                            relevant_cities.push(city_name.clone());
                        }
                    }
                }
            }
        }

        // If no city is found but a keyword is, assign to UNSPECIFIED_LOCATION
        if keyword_found && relevant_cities.is_empty() {
            println!("Keyword found in content, but no city, assigning to UNSPECIFIED_LOCATION.");
            relevant_cities.push("UNSPECIFIED_LOCATION".to_string());
        }
    }

    // If no keyword is found in both the title and content, do not add to the output
    if !keyword_found {
        println!("No keyword found in title or content, skipping.");
        return Ok(());
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

    // Update the shared news_by_city map
    let mut news_by_city_lock = news_by_city.lock().unwrap();
    for (city, news_item) in news_updates {
        news_by_city_lock.entry(city).or_insert_with(Vec::new).push(news_item);
    }

    Ok(())
}

// Running parallel processing for all entries
fn run_in_parallel(
    metadata: Vec<MetadataEntry>,
    keywords: HashSet<String>,
    city_map: HashMap<String, City>,
    fuzzy_threshold_title: f64,    // Fuzzy matching threshold for title
    fuzzy_threshold_content: f64,  // Strict matching threshold for content
    proximity_threshold: usize,    // Proximity threshold for content matching
    relaxed: bool                  // Enable relaxed mode
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
            proximity_threshold, // Pass proximity threshold
            relaxed,             // Pass relaxed mode
        );
        if let Err(err) = result {
            eprintln!("Error processing entry {}: {}", entry.filepath, err);
        }
    });

    // Collect the results from the HashMap into a Vec<NewsByCity>
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

// Main function
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let relaxed = args.contains(&"--relaxed".to_string());

    if args.len() < 2 {
        eprintln!("Usage: cargo run <keyword1> <keyword2> ... [--relaxed]");
        std::process::exit(1);
    }

    // Collect keywords from command line arguments
    let keywords: HashSet<String> = args[1..].iter()
        .filter(|&arg| arg != "--relaxed")
        .map(|k| k.to_lowercase())
        .collect();

    let city_data: Vec<City> = serde_json::from_str(&fs::read_to_string("data/cities/cities-poland.json")?)?;

    // Create a HashMap for cities for O(1) lookup
    let city_map: HashMap<String, City> = city_data.into_iter()
        .map(|city| (city.name.to_lowercase(), city))
        .collect();

    // Read metadata for processing
    let metadata: Vec<MetadataEntry> = serde_json::from_str(&fs::read_to_string("data/output/metadata.json")?)?;

    // Define fuzzy matching thresholds and proximity limits
    let fuzzy_threshold_title = 0.95;  // Stricter fuzzy matching for title
    let fuzzy_threshold_content = 0.95; // Stricter fuzzy matching for content
    let proximity_threshold = 3;       // Proximity of 5 words in content

    // Process entries in parallel
    let news_by_city = run_in_parallel(metadata, keywords, city_map, fuzzy_threshold_title, fuzzy_threshold_content, proximity_threshold, relaxed)?;

    // Write output to JSON file
    let output_json = serde_json::to_string_pretty(&news_by_city)?;
    fs::create_dir_all("data/mapped")?;
    fs::write("data/mapped/news_by_city.json", output_json)?;

    println!("Processing completed. Results saved to 'data/mapped/news_by_city.json'.");
    Ok(())
}
