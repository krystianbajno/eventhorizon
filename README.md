The News OSINT tool that allows operators to search for specific cities and keywords, then visualize events on a map. 

Example usage - track and map flooding events in Poland in September 2024.

It aggregates news and maps them to the geo map. Search for keywords and cities.

If the keyword in news could not be mapped to city, then the news will be mapped to "UNSPECIFIED_LOCATION".

How to:

### 1. Setup your news_sources_selected.txt
```
sources/news_sources_selected.txt
```

### 2. Retrieve cities database
```
cd collect_cities
npm install
node index.js
```

### 3. Run collection
```
bash collect.sh
```

### 4. Run process.py
```
python3 process.py keyword1 keyword2 keyword3
```

### 5. Get your output
Your output .json will be in `data/mapped/news_by_city.json`.

### 6. Open index.html and import the .json file
Open the index.html in your browser and import the .json file.