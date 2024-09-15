use regex::Regex;
use lazy_static::lazy_static;
use strsim::jaro_winkler;

lazy_static! {
    static ref WORD_SPLIT_REGEX: Regex = Regex::new(r"[^\w]+").unwrap();
}

pub fn split_content_into_words(content: &str) -> Vec<String> {
    WORD_SPLIT_REGEX
        .split(content)
        .map(|word| word.to_lowercase())
        .filter(|word| !word.is_empty())
        .collect()
}

pub fn fuzzy_match_city(city_name: &str, word: &str, threshold: f64) -> bool {
    jaro_winkler(city_name, word) >= threshold
}
