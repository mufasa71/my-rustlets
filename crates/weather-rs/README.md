# Weather-rs

This project is a simple weather application written in Rust.
It uses the weatherapi.com and rapidapi subscription to fetch
current weather data for a given city.

## Usage

Copy [example/weather_conditions.json](example/weather_conditions.json)
into `~/.config/weather-rs` directory.

Add *rapidapi key* into `~/.config/rapidapi.toml` (see [example/rapidapi.toml](example/rapidapi.toml)).

Then run the following command:

```bash

weather-rs -q "London"
```
