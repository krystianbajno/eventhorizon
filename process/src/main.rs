use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use strsim::jaro_winkler;
use anyhow::Result;
use std::sync::{Arc, Mutex};

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

fn split_content_into_words(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|word| !word.is_empty())
        .collect()
}

fn is_keyword_in_content(content_words: &[String], keywords: &[String], min_matches: usize) -> bool {
    for keyword in keywords {
        let match_count = content_words
            .iter()
            .filter(|word| jaro_winkler(word, keyword) >= 0.8)
            .count();
        if match_count >= min_matches {
            return true;
        }
    }
    false
}

fn match_cities_in_content(content_words: &[String], city_names: &[String], min_matches: usize) -> Vec<String> {
    city_names
        .iter()
        .filter_map(|city_name| {
            let match_count = content_words
                .iter()
                .filter(|word| jaro_winkler(word, city_name) >= 0.8)
                .count();
            if match_count >= min_matches {
                Some(city_name.clone())
            } else {
                None
            }
        })
        .collect()
}

fn process_entry(
    entry: &MetadataEntry,
    keywords: &[String],
    city_data: &Vec<City>,
    news_by_city: Arc<Mutex<Vec<NewsByCity>>>,
    min_matches: usize,
) -> Result<()> {
    if Path::new(&entry.filepath).exists() {
        let content = fs::read_to_string(&entry.filepath)?;
        let content_words = split_content_into_words(&content);

        println!("{:?}", entry.filepath);

        if is_keyword_in_content(&content_words, keywords, min_matches) {
            let matched_cities: Vec<String> = match_cities_in_content(
                &content_words,
                &city_data.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
                min_matches,
            );

            let mut news_by_city = news_by_city.lock().unwrap();

            if !matched_cities.is_empty() {
                for city_name in matched_cities {
                    let city_info = city_data
                        .iter()
                        .find(|c| c.name == city_name)
                        .expect("City not found");
                    let city_coords = Some(city_info.loc.coordinates.clone());

                    if let Some(city_entry) = news_by_city.iter_mut().find(|c| c.city == city_name) {
                        city_entry.news.push(NewsItem {
                            title: entry.title.clone(),
                            link: entry.url.clone(),
                            filepath: entry.filepath.clone(),
                            collection_date: entry.collection_date.clone(),
                        });
                    } else {
                        news_by_city.push(NewsByCity {
                            city: city_name.clone(),
                            coordinates: city_coords,
                            news: vec![NewsItem {
                                title: entry.title.clone(),
                                link: entry.url.clone(),
                                filepath: entry.filepath.clone(),
                                collection_date: entry.collection_date.clone(),
                            }],
                        });
                    }
                }
            } else {
                if let Some(city_entry) = news_by_city.iter_mut().find(|c| c.city == "UNSPECIFIED_LOCATION") {
                    city_entry.news.push(NewsItem {
                        title: entry.title.clone(),
                        link: entry.url.clone(),
                        filepath: entry.filepath.clone(),
                        collection_date: entry.collection_date.clone(),
                    });
                } else {
                    news_by_city.push(NewsByCity {
                        city: "UNSPECIFIED_LOCATION".to_string(),
                        coordinates: None,
                        news: vec![NewsItem {
                            title: entry.title.clone(),
                            link: entry.url.clone(),
                            filepath: entry.filepath.clone(),
                            collection_date: entry.collection_date.clone(),
                        }],
                    });
                }
            }
        }
    }
    Ok(())
}

fn run_in_parallel(
    metadata: Vec<MetadataEntry>,
    keywords: Vec<String>,
    city_data: Vec<City>,
    min_matches: usize,
) -> Result<Vec<NewsByCity>> {
    let news_by_city = Arc::new(Mutex::new(Vec::new()));

    metadata.par_iter().for_each(|entry| {
        let result = process_entry(entry, &keywords, &city_data, Arc::clone(&news_by_city), min_matches);
        if let Err(err) = result {
            eprintln!("Error processing entry {}: {}", entry.filepath, err);
        }
    });

    let final_result = Arc::try_unwrap(news_by_city)
        .unwrap()
        .into_inner()
        .unwrap();
    Ok(final_result)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run <keyword1> <keyword2> ...");
        std::process::exit(1);
    }

    let keywords: Vec<String> = args[1..].iter().map(|k| k.to_lowercase()).collect();

    let city_data: Vec<City> =
        serde_json::from_str(&fs::read_to_string("data/cities/cities-poland.json")?)?;

    let metadata: Vec<MetadataEntry> =
        serde_json::from_str(&fs::read_to_string("data/output/metadata.json")?)?;

    let min_matches = 2;

    let news_by_city = run_in_parallel(metadata, keywords, city_data, min_matches)?;

    let output_json = serde_json::to_string_pretty(&news_by_city)?;
    fs::create_dir_all("data/mapped")?;
    fs::write("data/mapped/news_by_city.json", output_json)?;

    println!("News mapped to cities and saved to 'data/mapped/news_by_city.json'");
    Ok(())
}
