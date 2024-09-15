use anyhow::Result;
use crate::models::{City, MetadataEntry, NewsByCity};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn load_cities(filepath: &str) -> Result<HashMap<String, City>> {
    let city_data: Vec<City> = serde_json::from_str(&fs::read_to_string(filepath)?)?;
    let city_map = city_data.into_iter().map(|city| (city.name.to_lowercase(), city)).collect();
    Ok(city_map)
}

pub fn load_metadata(filepath: &str) -> Result<Vec<MetadataEntry>> {
    let metadata: Vec<MetadataEntry> = serde_json::from_str(&fs::read_to_string(filepath)?)?;
    Ok(metadata)
}

pub fn write_output(news_by_city: &Vec<NewsByCity>, output_path: &str) -> Result<()> {
    let output_json = serde_json::to_string_pretty(news_by_city)?;
    fs::create_dir_all(Path::new(output_path).parent().unwrap())?;
    fs::write(output_path, output_json)?;
    Ok(())
}

pub fn mmap_file(filepath: &str) -> Result<String> {
    use memmap2::Mmap;
    use std::fs::File;

    let file = File::open(filepath)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = std::str::from_utf8(&mmap).unwrap_or("").to_string();
    Ok(content)
}
