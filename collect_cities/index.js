const fs = require("fs")
const cities = require('all-the-cities');

fs.writeFileSync("../data/cities.json", JSON.stringify(cities))
fs.writeFileSync("../data/cities-poland.json", JSON.stringify(cities.filter(i => i.country.match("PL"))))