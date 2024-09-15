import os
import json
import sys
from fuzzywuzzy import fuzz
from collections import defaultdict
from datetime import datetime

if len(sys.argv) < 2:
    print("Usage: python3 process.py <keyword1> <keyword2> ...")
    sys.exit(1)

keywords = sys.argv[1:]

with open("data/cities/cities-poland.json", encoding="utf-8") as f:
    cities = json.load(f)

city_data = {city["name"]: city for city in cities}

output_dir = "data/output/scraped"
metadata_file = "data/output/metadata.json"

news_by_city = defaultdict(lambda: {"coordinates": None, "news": []})

def is_keyword_in_content(content, keywords):
    for keyword in keywords:
        if keyword.lower() in content.lower():
            return True
    return False

def match_cities_in_content(content, city_names):
    matched_cities = []
    for city_name in city_names:
        if fuzz.partial_ratio(city_name.lower(), content.lower()) >= 60:
            matched_cities.append(city_name)
    return matched_cities

collection_date = datetime.now().strftime('%Y-%m-%d %H:%M:%S')

if not os.path.exists(metadata_file):
    print(f"Metadata file {metadata_file} not found.")
    sys.exit(1)

with open(metadata_file, "r", encoding="utf-8") as meta_file:
    metadata = json.load(meta_file)

for entry in metadata:
    filepath = entry["filepath"]
    title = entry["title"]
    url = entry["url"]

    if os.path.exists(filepath):
        with open(filepath, "r", encoding="utf-8") as content_file:
            content = content_file.read()

            if is_keyword_in_content(content, keywords):
                matched_cities = match_cities_in_content(content, city_data.keys())

                if matched_cities:
                    for matched_city in matched_cities:
                        city_info = city_data[matched_city]
                        city_coords = city_info["loc"]["coordinates"]

                        news_by_city[matched_city]["coordinates"] = city_coords
                        news_by_city[matched_city]["news"].append({
                            "Title": title,
                            "Link": url,
                            "File Path": filepath,
                            "Collection Date": collection_date
                        })
                else:
                    news_by_city["UNSPECIFIED_LOCATION"]["coordinates"] = None
                    news_by_city["UNSPECIFIED_LOCATION"]["news"].append({
                        "Title": title,
                        "Link": url,
                        "File Path": filepath,
                        "Collection Date": collection_date
                    })

output_json = []
for city, data in news_by_city.items():
    output_json.append({
        "city": city,
        "coordinates": data["coordinates"],
        "news": data["news"]
    })

os.makedirs("data/mapped", exist_ok=True)
with open("data/mapped/news_by_city.json", "w", encoding="utf-8") as json_output_file:
    json.dump(output_json, json_output_file, ensure_ascii=False, indent=4)

print("News mapped to cities and saved to 'data/mapped/news_by_city.json'")
