# eventhorizon

<img src="https://raw.githubusercontent.com/krystianbajno/eventhorizon/main/images/image.png"/>

The versatile OSINT tool that allows operators to search for specific cities and keywords, then visualize events on a map. 

It aggregates information and maps it to the geo map. Fuzzy search for keywords and cities.

# Example usages 

- **Track natural disasters**: Visualize events like flooding in Poland during September 2024.
- **Monitor war events**: Track and map incidents in warzones.
- **Headline analysis**: Plot keyword mentions from news article titles on a map.
- **Keyword mapping**: Search websites for terms and automatically map them to locations.
- **Job hunting**: Scrape job listings (e.g., "remote Python developer") and map them.
- **Track protests**: Aggregate social or political event mentions and map them.
- **Monitor supply chain issues**: Map disruptions like port strikes or factory shutdowns.

And more.

If the keyword in event could not be mapped to city, then the event will be mapped to "UNSPECIFIED_LOCATION".

# Intelligence Lifecycle
1. Direction
2. Collection
3. Processing and exploitation
4. Analysis
5. Dissemination

# How to:

### 0. Install dependencies
- Install katana - https://github.com/projectdiscovery/katana
- Install playwright - https://playwright.dev/python/docs/intro 
- Install node - https://nodejs.org/en
- Install python - https://www.python.org/downloads/
- Install rust - https://github.com/rust-lang/rust
- Run `pip install -r requirements.txt`

### 1. Setup your news_sources_selected.txt
Configure the sources from which you want to retrieve information by editing the news_sources_selected.txt file:
```
sources/news_sources_selected.txt
```

### 2. Retrieve cities database
Run the following commands to install the necessary dependencies and retrieve the database of cities:
```
cd collect_cities
npm install
node index.js
```

### 3. Run collection `collect.py`
Execute the Python script to collect data from your chosen sources:
```
python collect.py
```

### 4. Run processing `processing`
Compile the processing module in Rust, place it in root project directory, then run it to parse the data:

```bash
./processing keyword1 keyword2 keyword3 # parse only titles
./processing keyword1 keyword2 --parse-content # parse content too
```

Precompiled Mach-O binary for MacOS M1 can be found in this repository.

### 5. Retrieve the Output 
The tool will generate a JSON file containing the mapped data. This file will be stored at:
`data/mapped/news_by_city.json`.

### 6. Visualize the Data 
Open index.html in your browser and import the generated JSON file to view the events on a map.
