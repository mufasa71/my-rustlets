use std::time::Duration;

use clap::Parser;
use dbus::{
    arg::{RefArg, Variant},
    nonblock::{Proxy, SyncConnection},
};
use dbus_tokio::connection;
use log::{Level, info};
use mqtt_to_dbus::{DBus, MqttToDbus, cli::Cli};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Payload {
    command: String,
}

const PLAYER_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

struct DBusPCControl {}

impl DBusPCControl {
    async fn sleep(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (resource, conn) = connection::new_system_sync()?;
        tokio::spawn(async move {
            let _ = resource.await;
        });

        let logind = Proxy::new(
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            Duration::from_secs(5),
            conn,
        );
        let _: () = logind
            .method_call("org.freedesktop.login1.Manager", "Suspend", (true,))
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl mqtt_to_dbus::Handler for DBusPCControl {
    async fn handle(
        &self,
        bytes: &[u8],
        proxy: Proxy<'_, std::sync::Arc<SyncConnection>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload: Payload = serde_json::from_slice(bytes)?;
        info!("Received payload: {:?}", payload);

        let command = payload.command.as_str();

        if command == "sleep" {
            self.sleep().await?;

            return Ok(());
        }

        let (names,): (Vec<String>,) = proxy
            .method_call("org.freedesktop.DBus", "ListNames", ())
            .await?;

        for name in names {
            if !name.starts_with("org.mpris.MediaPlayer2.") {
                continue;
            }

            let player = Proxy::new(
                name.as_str(),
                "/org/mpris/MediaPlayer2",
                Duration::from_secs(2),
                proxy.connection.clone(),
            );

            let (value,): (Variant<Box<dyn RefArg>>,) = player
                .method_call(
                    "org.freedesktop.DBus.Properties",
                    "Get",
                    (PLAYER_INTERFACE, "PlaybackStatus"),
                )
                .await?;

            let status = value.0.as_str().unwrap_or_default();

            if status == "Paused" || status == "Playing" {
                match command {
                    "play" => {
                        let _: () = player.method_call(PLAYER_INTERFACE, "Play", ()).await?;

                        info!("Sent Play command to {}", name);
                    }
                    "pause" => {
                        let _: () = player.method_call(PLAYER_INTERFACE, "Pause", ()).await?;
                        info!("Sent Pause command to {}", name);
                    }
                    "nextsong" => {
                        let _: () = player.method_call(PLAYER_INTERFACE, "Next", ()).await?;
                        info!("Sent Next command to {}", name);
                    }
                    "prevsong" => {
                        let _: () = player.method_call(PLAYER_INTERFACE, "Previous", ()).await?;
                        info!("Sent Previous command to {}", name);
                    }
                    _ => {
                        info!("Unknown command: {}", payload.command);
                    }
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(Level::Info)?;
    let cli = Cli::parse();
    let handler = DBusPCControl {};
    let mqtt_to_dbus = MqttToDbus::new(
        cli,
        DBus {
            dest: "org.freedesktop.DBus".to_string(),
            path: "/org/freedesktop/DBus".to_string(),
            handler,
        },
    );

    mqtt_to_dbus.run().await?;

    Ok(())
}
