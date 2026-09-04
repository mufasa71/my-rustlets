use std::time::Duration;

use dbus::nonblock;
use dbus_tokio::connection;
use log::{Level, info, warn};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use tokio::{
    io::AsyncWriteExt,
    net::UnixStream,
    signal::unix::{SignalKind, signal},
};

// Systemd service file for this program:
// Restart=on-failure
// RestartSec=5s
// EnvironmentFile=/etc/default/lux-rs
// After=darkman.service
// PartOf=graphical-session.target

// Example message from zigbee2mqtt:
// {"battery":80,"illuminance":5,"linkquality":160,"voltage":2800}
#[derive(Deserialize)]
struct Message {
    illuminance: Option<u32>,
}

async fn send_command(socket_path: &str, command: &str) -> std::io::Result<()> {
    let mut stream = UnixStream::connect(socket_path).await?;

    stream.write_all(format!("{command}\n").as_bytes()).await?;

    stream.shutdown().await
}

fn env(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|e| format!("{key}: {e}"))
}

// observed max raw illuminance for this sensor
const MAX_ILLUMINANCE: f64 = 338.0;
// dead zone between toggle to avoid sending the same command repeatedly
const LIGHT_THRESHOLD: f64 = 0.35;
const DARK_THRESHOLD: f64 = 0.30;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(Level::Info)?;
    // mqtt setup
    let runtime_dir = env("XDG_RUNTIME_DIR")?;
    let ha_password = env("HA_PASSWORD")?;
    let ha_user = env("HA_USERNAME")?;
    let zigbee_lux_topic = env("HA_ZIGBEE_LUX_TOPIC")?;
    let socket_path = format!("{runtime_dir}/darkman/control.sock");
    // TODO: make host:port configurable
    let mut mqttoptions = MqttOptions::new("lux-rs", "homeassistant", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(60));
    mqttoptions.set_credentials(ha_user, ha_password);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // signal handling
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    // dbus setup
    let (resource, conn) = connection::new_session_sync()?;
    let mut dbus_resource = tokio::spawn(resource);
    let proxy = nonblock::Proxy::new("rs.i3status", "/lux", Duration::from_secs(2), conn);

    loop {
        let event = tokio::select! {
            err = &mut dbus_resource =>{
                warn!("lost D-Bus connection: {err:?}");
                return Err("lost D-Bus connection".into());
            }
            _ = sigint.recv()=> {
                info!("SIGINT received, shutting down");
                break;
            }
            _ = sigterm.recv()=> {
                info!("SIGTERM received, shutting down");
                break;
            }
            event = eventloop.poll() => event,
        };
        match event {
            Ok(event) => match event {
                Event::Incoming(Packet::ConnAck(_)) => {
                    if let Err(e) = client.subscribe(&zigbee_lux_topic, QoS::AtMostOnce).await {
                        warn!("Error subscribing to topic: {e}");
                    } else {
                        info!("Subscribed to topic: {zigbee_lux_topic}");
                    }
                }
                Event::Incoming(Packet::Publish(publish)) => {
                    match serde_json::from_slice::<Message>(publish.payload.as_ref()) {
                        Ok(Message {
                            illuminance: Some(lux),
                        }) => {
                            let illuminance = f64::from(lux) / MAX_ILLUMINANCE;
                            let lux_percent = illuminance * 100.0;
                            info!("Lux: {:.0}%", lux_percent);

                            let command = if illuminance < DARK_THRESHOLD {
                                Some("set dark")
                            } else if illuminance > LIGHT_THRESHOLD {
                                Some("set light")
                            } else {
                                None
                            };

                            if let Some(command) = command {
                                match send_command(&socket_path, command).await {
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
                        }
                        Ok(_) => {}
                        Err(e) => warn!("Could not parse payload: {e}"),
                    }
                }
                _ => {}
            },
            Err(e) => {
                warn!("{e}");
            }
        }
    }

    Ok(())
}
