use clap::Parser;
use dbus::nonblock::{Proxy, SyncConnection};
use log::{Level, info, warn};
use mqtt_to_dbus::{DBus, MqttToDbus, cli::Cli};
use serde::Deserialize;
use std::str::FromStr;
use strum::{Display, EnumString};

#[derive(Deserialize, Debug)]
struct Payload {
    temp: f32,
    is_day: bool,
    // humidity: f32,
    current_state: String,
}

struct DBusRoomTemperature {
    dbus_interface: String,
}

#[async_trait::async_trait]
impl mqtt_to_dbus::Handler for DBusRoomTemperature {
    async fn handle(
        &self,
        bytes: &[u8],
        proxy: Proxy<'_, std::sync::Arc<SyncConnection>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload: Payload = serde_json::from_slice(bytes)?;
        info!("Received payload: {:?}", payload);

        let dbus_interface = self.dbus_interface.clone();

        if let Err(e) = proxy
            .method_call::<(), _, _, _>(
                &dbus_interface,
                "SetText",
                (payload.temp.to_string(), String::new()),
            )
            .await
        {
            warn!("{dbus_interface} SetText failed: {e}");
        }
        if let Err(e) = proxy
            .method_call::<(), _, _, _>(&dbus_interface, "SetIcon", (get_icon(&payload),))
            .await
        {
            warn!("{dbus_interface} SetIcon failed: {e}");
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(Level::Info)?;
    let cli = Cli::parse();

    let mqtt_to_dbus = MqttToDbus::new(
        cli,
        DBus {
            dest: "rs.i3status.bottom".to_string(),
            path: "/sensor_forecast".to_string(),
            handler: DBusRoomTemperature {
                dbus_interface: "rs.i3status.custom".to_string(),
            },
        },
    );

    mqtt_to_dbus.run().await?;

    Ok(())
}

#[derive(EnumString, Display)]
enum StateToCode {
    #[strum(serialize = "cloudy", to_string = "weather_clouds")]
    WeatherClouds,
    #[strum(serialize = "fog", to_string = "weather_fog")]
    Fog,
    #[strum(serialize = "partlycloudy", to_string = "weather_partly_cloudy")]
    PartlyCloudy,
    #[strum(serialize = "sun", serialize = "clear-night", to_string = "weather_sun")]
    Sun,
    #[strum(serialize = "snow", to_string = "weather_snow")]
    Snow,
}

fn get_icon(payload: &Payload) -> String {
    let state_code = StateToCode::from_str(&payload.current_state);

    match state_code {
        Ok(code) => {
            if payload.is_day {
                return code.to_string();
            }

            format!("{code}_night")
        }
        Err(_) => String::from(""),
    }
}
