mod models;
mod file_io;
mod text_processing;
mod processing;
mod parallel_processing;

use anyhow::Result;
use std::collections::HashSet;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let parse_content = args.contains(&"--parse-content".to_string());

    if args.len() < 2 {
        eprintln!("Usage: cargo run <keyword1> <keyword2> ... [--parse-content]");
        std::process::exit(1);
    }

    // Collect keywords from command line arguments
    let keywords: HashSet<String> = args[1..].iter()
        .filter(|&arg| arg != "--parse-content")
        .map(|k| k.to_lowercase())
        .collect();

    // Load cities and metadata
    let city_map = file_io::load_cities("data/cities/cities-poland.json")?;
    let metadata = file_io::load_metadata("data/output/metadata.json")?;

    // Define fuzzy matching thresholds and proximity limits
    let fuzzy_threshold_title = 0.95;
    let fuzzy_threshold_content = 0.95;
    let proximity_threshold = 3;

    let news_by_city = parallel_processing::run_in_parallel(
        metadata,
        keywords,
        city_map,
        fuzzy_threshold_title,
        fuzzy_threshold_content,
        proximity_threshold,
        parse_content
    )?;

    file_io::write_output(&news_by_city, "data/mapped/news_by_city.json")?;

    println!("Processing completed. Results saved to 'data/mapped/news_by_city.json'.");
    Ok(())
}
