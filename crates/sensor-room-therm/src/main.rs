use clap::Parser;
use dbus::nonblock::{Proxy, SyncConnection};
use log::{Level, warn};
use mqtt_to_dbus::{DBus, MqttToDbus, cli::Cli};
use serde::Deserialize;

#[derive(Deserialize)]
struct Payload {
    temp: Option<f32>,
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
        if let Some(temp) = payload.temp {
            log::info!("Received data: {}", temp);

            let dbus_interface = self.dbus_interface.clone();

            if let Err(e) = proxy
                .method_call::<(), _, _, _>(
                    &dbus_interface,
                    "SetText",
                    (temp.to_string(), String::new()),
                )
                .await
            {
                warn!("{dbus_interface} SetText failed: {e}");
            }
        } else {
            warn!("Failed to parse message");
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
            path: "/sensor_room_therm".to_string(),
            handler: DBusRoomTemperature {
                dbus_interface: "rs.i3status.custom".to_string(),
            },
        },
    );

    mqtt_to_dbus.run().await?;

    Ok(())
}
