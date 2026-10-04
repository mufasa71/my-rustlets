pub mod cli;

use std::{sync::Arc, time::Duration};

use dbus::{
    nonblock,
    nonblock::{Proxy, SyncConnection},
};
use dbus_tokio::connection;
use log::{info, warn};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::signal::unix::{SignalKind, signal};
use uuid::Uuid;

use crate::cli::Cli;

#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    async fn handle(
        &self,
        bytes: &[u8],
        proxy: Proxy<'_, Arc<SyncConnection>>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct DBus<H: Handler> {
    pub dest: String,
    pub path: String,
    pub handler: H,
}

pub struct MqttToDbus<H: Handler> {
    mqttoptions: MqttOptions,
    mqtt_topic: String,
    dbus: DBus<H>,
}

impl<H: Handler> MqttToDbus<H> {
    pub fn new(cli: Cli, dbus: DBus<H>) -> Self {
        let mqtt_topic = cli.ha_mqtt_topic;
        let mut mqttoptions = MqttOptions::new(Uuid::new_v4(), cli.hostname, cli.port);

        mqttoptions.set_keep_alive(Duration::from_secs(60));
        mqttoptions.set_credentials(cli.ha_username, cli.ha_password);

        MqttToDbus {
            mqttoptions,
            mqtt_topic,
            dbus,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Implement the logic to connect to MQTT, subscribe to topics, and forward messages to D-Bus
        let (client, mut eventloop) = AsyncClient::new(self.mqttoptions.clone(), 10);
        let (resource, conn) = connection::new_session_sync()?;
        let mut dbus_resource = tokio::spawn(resource);

        let proxy = nonblock::Proxy::new(
            self.dbus.dest.clone(),
            self.dbus.path.clone(),
            Duration::from_secs(2),
            conn,
        );

        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;

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
                        let mqtt_topic = self.mqtt_topic.clone();

                        if let Err(e) = client.subscribe(&mqtt_topic, QoS::AtMostOnce).await {
                            warn!("Error subscribing to topic: {e}");
                        } else {
                            info!("Subscribed to topic: {mqtt_topic}");
                        }
                    }
                    Event::Incoming(Packet::Publish(publish)) => {
                        self.dbus
                            .handler
                            .handle(publish.payload.as_ref(), proxy.clone())
                            .await?;
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
}
