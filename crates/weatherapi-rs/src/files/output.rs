use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

use crate::files::Weather;

#[derive(Serialize, Deserialize, Debug)]
struct WeatherCode {
    code: i32,
    day: String,
    night: String,
    icon: i32,
    emoji: String,
    night_emoji: String,
}

fn get_icon(weather: &Weather) -> String {
    let mut config_dir = dirs::config_dir().expect("Could not find config directory");
    config_dir.push("weatherapi-rs/weather_conditions.json");
    let file = File::open(config_dir).expect("Could not open file weather_conditions.json");
    let reader = BufReader::new(file);
    let weather_codes: Vec<WeatherCode> =
        serde_json::from_reader(reader).expect("Could not parse weather_conditions.json");
    let code = weather_codes
        .iter()
        .find(|&item| item.code == weather.current.condition.code);

    match code {
        Some(code) => {
            if weather.current.is_day == 1 {
                code.emoji.clone()
            } else {
                code.night_emoji.clone()
            }
        }
        None => String::from("❓"),
    }
}

pub fn output_compact(weather: &Weather) -> String {
    let mut output = String::new();
    output.push_str(&format!("{} ", get_icon(weather)));
    output.push_str(&format!("{}°C", weather.current.temp_c));
    output.push_str(&format!(" {}", weather.current.condition.text));

    if let Some(ref air_quality) = weather.current.air_quality {
        output.push_str(&format!(" (AQI: {})", air_quality.us_epa_index));
    }

    output
}
