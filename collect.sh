#!/bin/bash

NEWS_SOURCES="news_sources.txt"

OUTPUT_DIR="scraped_news"
mkdir -p "$OUTPUT_DIR"

scrape_news() {
    local url=$1
    local filename=$(echo "$url" | sed 's/[^a-zA-Z0-9]/_/g')

    echo "Scraping $url"
    katana -u "$url" -d 1 --silent | grep -Eo "(http|https)://[a-zA-Z0-9./?=_-]*" > "$OUTPUT_DIR/${filename}_urls.txt"

    for news_url in $(cat "$OUTPUT_DIR/${filename}_urls.txt"); do
        page_content=$(curl -s "$news_url")
        title=$(echo "$page_content" | grep -oP '(?<=<title>)(.*)(?=</title>)')
        body=$(echo "$page_content" | sed -n '/<body/,/<\/body>/p' | tr '\n' ' ' | sed 's/<[^>]*>//g' | sed 's/[[:space:]]\+/ /g')

        echo "Title: $title" >> "$OUTPUT_DIR/${filename}_content.txt"
        echo "URL: $news_url" >> "$OUTPUT_DIR/${filename}_content.txt"
        echo "Content: $body" >> "$OUTPUT_DIR/${filename}_content.txt"
        echo "------------------------" >> "$OUTPUT_DIR/${filename}_content.txt"
    done
}

while IFS= read -r url; do
    scrape_news "$url"
done < "$NEWS_SOURCES"

echo "Scraping complete. Data saved in $OUTPUT_DIR."
