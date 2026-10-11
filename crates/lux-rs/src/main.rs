mod cli;

use clap::Parser;
use dbus::nonblock::{Proxy, SyncConnection};
use log::{Level, info, warn};
use mqtt_to_dbus::{DBus, MqttToDbus};
use serde::Deserialize;
use tokio::{io::AsyncWriteExt, net::UnixStream};

use crate::cli::LuxCli;

// Systemd service file for this program:
// Restart=on-failure
// RestartSec=5s
// EnvironmentFile=/etc/default/lux-rs
// After=darkman.service
// PartOf=graphical-session.target

// Example message from zigbee2mqtt:
// {"battery":80,"illuminance":5,"linkquality":160,"voltage":2800}
#[derive(Deserialize, Debug)]
struct Payload {
    illuminance: u32,
}

async fn send_command(socket_path: &str, command: &str) -> std::io::Result<()> {
    let mut stream = UnixStream::connect(socket_path).await?;

    stream.write_all(format!("{command}\n").as_bytes()).await?;

    stream.shutdown().await
}

struct DBusLux {
    max_illuminance: f64,
    dark_threshold: f64,
    light_threshold: f64,
    socket_path: String,
}

#[async_trait::async_trait]
impl mqtt_to_dbus::Handler for DBusLux {
    async fn handle(
        &self,
        bytes: &[u8],
        proxy: Proxy<'_, std::sync::Arc<SyncConnection>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload: Payload = serde_json::from_slice(bytes)?;
        info!("Received payload: {:?}", payload);

        let illuminance = f64::from(payload.illuminance) / self.max_illuminance;
        let lux_percent = illuminance * 100.0;
        info!("Lux: {:.0}%", lux_percent);

        let command = if illuminance < self.dark_threshold {
            Some("set dark")
        } else if illuminance > self.light_threshold {
            Some("set light")
        } else {
            None
        };

        if let Some(command) = command {
            match send_command(&self.socket_path, command).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("darkman socket write failed: {e}");
                }
            }
        }
        if let Err(e) = proxy
            .method_call::<(), _, _, _>(
                "rs.i3status.custom",
                "SetText",
                (format!("{lux_percent:.0}%"), String::new()),
            )
            .await
        {
            warn!("i3status SetText failed: {e}");
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(Level::Info)?;
    let cli = LuxCli::parse();
    let handler = DBusLux {
        max_illuminance: cli.max_illuminance,
        dark_threshold: cli.dark_threshold,
        light_threshold: cli.light_threshold,
        socket_path: format!("{}/darkman/control.sock", cli.runtime_dir),
    };
    let mqtt_to_dbus = MqttToDbus::new(
        cli.mqtt,
        DBus {
            dest: "rs.i3status".to_string(),
            path: "/lux".to_string(),
            handler,
        },
    );

    mqtt_to_dbus.run().await?;

    Ok(())
}
