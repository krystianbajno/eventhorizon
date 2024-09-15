import os
import hashlib
import json
import re
import subprocess
from datetime import datetime
import time
from playwright.sync_api import sync_playwright

NEWS_SOURCES = "sources/news_sources_selected.txt"
OUTPUT_DIR = "data/output"
ALL_URLS_FILE = os.path.join(OUTPUT_DIR, "all_news_urls.txt")
SCRAPED_DIR = os.path.join(OUTPUT_DIR, "scraped")
METADATA_FILE = os.path.join(OUTPUT_DIR, "metadata.json")
SCRAPING_DEPTH = 1
EXCLUDE_EXT = [".js", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".woff", ".woff2", ".tiff", ".ico"]

os.makedirs(SCRAPED_DIR, exist_ok=True)

def run_katana_for_urls(news_sources, all_urls_file):
    katana_cmd = [
        "katana",
        "-list", news_sources,
        "-d", str(SCRAPING_DEPTH),
        "--exclude", ".js,.css,.png,.jpg,.jpeg,.gif,.svg,.woff,.woff2,.tiff,.ico"
    ]
    try:
        with open(all_urls_file, "w") as f:
            subprocess.run(katana_cmd, stdout=f, check=True)
        print(f"URLs collected and saved in {all_urls_file}")
    except subprocess.CalledProcessError as e:
        print(f"Failed to run katana for URLs: {e}")
        exit(1)

def download_full_page_with_js(url, output_file):
    with sync_playwright() as p:
        try:
            browser = p.chromium.launch(headless=True)
            page = browser.new_page()
            page.goto(url)
            page_content = page.content()
            with open(output_file, "w", encoding="utf-8") as f:
                f.write(page_content)
            print(f"Downloaded and saved full content for {url}")
            browser.close()
        except Exception as e:
            print(f"Error parsing {url}: {e}")

def is_already_processed(url, list_of_scraped_dir_files):
    filename = hash_url(url)
    output_file = f"{filename}_content.html"
    
    return output_file in list_of_scraped_dir_files
        

def should_exclude_url(url):
    return any(url.endswith(ext) for ext in EXCLUDE_EXT)

def run_playwright_for_content(urls_file, scraped_dir, metadata):
    with open(urls_file, "r") as f:
        urls = [url.strip() for url in f if url.strip()]

    collection_date = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
    count = 0
    for url in urls:
        try:
            url_files = os.listdir(scraped_dir)
            if is_already_processed(url, url_files):
                print(f"Skipping URL due already processed. {url}")
                continue
            
            if should_exclude_url(url):
                print(f"Skipping URL due to excluded extension: {url}")
                continue
            
            count = count + 1
            filename = hash_url(url)
            output_file = os.path.join(scraped_dir, f"{filename}_content.html")

            download_full_page_with_js(url, output_file)
            
            with open(output_file, "r", encoding="utf-8") as f:
                content = f.read()
                title = extract_title(content)
            
            
            metadata.append({
                "filepath": output_file,
                "title": title,
                "url": url,
                "collection_date": collection_date
            })
        
            with open(METADATA_FILE, "w", encoding="utf-8") as json_file:
                json.dump(metadata, json_file, ensure_ascii=False, indent=4)
            print(f"Metadata saved to {METADATA_FILE}")
            
        except Exception as e:
            print(f"Whoops, something happened with: {url} - {e}")
            time.sleep(10)
        
    with open(METADATA_FILE, "w", encoding="utf-8") as json_file:
        json.dump(metadata, json_file, ensure_ascii=False, indent=4)
    print(f"Metadata saved to {METADATA_FILE}")

def hash_url(url):
    return hashlib.md5(url.encode()).hexdigest()

def extract_title(content):
    match = re.search(r"<title>(.*?)</title>", content)
    return match.group(1) if match else "No Title Found"

def main():
    METADATA = []

    if os.path.exists(ALL_URLS_FILE):
        choice = input(f"{ALL_URLS_FILE} exists. Do you want to start over and run katana again to collect URLs? (y/n): ")
        if choice.lower() == 'y':
            os.remove(ALL_URLS_FILE)
            run_katana_for_urls(NEWS_SOURCES, ALL_URLS_FILE)
        else:
            print(f"Using existing URLs from {ALL_URLS_FILE}")
    else:
        run_katana_for_urls(NEWS_SOURCES, ALL_URLS_FILE)
        
    
    if os.path.exists(METADATA_FILE):
        with open(METADATA_FILE, "r") as metadata_file:
            METADATA = json.loads(metadata_file.read())

    run_playwright_for_content(ALL_URLS_FILE, SCRAPED_DIR, METADATA)

    print("Finished")
    
if __name__ == "__main__":
    main()
