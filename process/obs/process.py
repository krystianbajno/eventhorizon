import os
import json
import sys
from fuzzywuzzy import fuzz
from multiprocessing import Pool, Manager

def is_keyword_in_content(content, keywords):
    content_lower = content.lower()
    for keyword in keywords:
        if fuzz.partial_ratio(keyword, content_lower) >= 60:
            return True
    return False

def match_cities_in_content(content, city_names):
    content_lower = content.lower()
    return [city_name for city_name in city_names if fuzz.partial_ratio(city_name, content_lower) >= 60]

def process_entry(entry, keywords, city_data, news_by_city):
    filepath = entry["filepath"]
    title = entry["title"]
    url = entry["url"]
    collection_date = entry["collection_date"]

    if os.path.exists(filepath):
        with open(filepath, "r", encoding="utf-8") as content_file:
            content = content_file.read()
            print(f"+ {filepath}")

            if is_keyword_in_content(content, keywords):
                matched_cities = match_cities_in_content(content, city_data.keys())

                if matched_cities:
                    for matched_city in matched_cities:
                        city_info = city_data[matched_city]
                        city_coords = city_info["loc"]["coordinates"]

                        news_by_city[matched_city]["coordinates"] = city_coords
                        news_by_city[matched_city]["news"].append({
                            "title": title,
                            "link": url,
                            "filepath": filepath,
                            "collection_date": collection_date
                        })
                else:
                    news_by_city["UNSPECIFIED_LOCATION"]["coordinates"] = None
                    news_by_city["UNSPECIFIED_LOCATION"]["news"].append({
                        "title": title,
                        "link": url,
                        "filepath": filepath,
                        "collection_date": collection_date
                    })
    
def run_in_parallel(metadata, keywords, city_data):
    def default_news_dict():
        return {"coordinates": None, "news": []}

    with Manager() as manager:
        news_by_city = manager.dict()

        for city_name in city_data.keys():
            news_by_city[city_name] = default_news_dict()

        news_by_city["UNSPECIFIED_LOCATION"] = default_news_dict()

        with Pool(os.cpu_count()) as pool:
            pool.starmap(process_entry, [(entry, keywords, city_data, news_by_city) for entry in metadata])

        return {key: dict(value) for key, value in news_by_city.items()}

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 process.py <keyword1> <keyword2> ...")
        sys.exit(1)

    keywords = set([k.lower() for k in sys.argv[1:]])

    with open("data/cities/cities-poland.json", encoding="utf-8") as f:
        cities = json.load(f)

    city_data = {city["name"].lower(): city for city in cities}

    metadata_file = "data/output/metadata.json"

    if not os.path.exists(metadata_file):
        print(f"Metadata file {metadata_file} not found.")
        sys.exit(1)

    with open(metadata_file, "r", encoding="utf-8") as meta_file:
        metadata = json.load(meta_file)

    news_by_city = run_in_parallel(metadata, keywords, city_data)

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
