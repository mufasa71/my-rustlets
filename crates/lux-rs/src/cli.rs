use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(long, default_value = "homeassistant", env = "HOSTNAME")]
    pub hostname: String,
    #[arg(short, default_value = "1883", env = "PORT")]
    pub port: u16,
    #[arg(long, env = "HA_USERNAME")]
    pub ha_username: String,
    #[arg(long, env = "HA_PASSWORD")]
    pub ha_password: String,
    #[arg(long, env = "HA_ZIGBEE_LUX_TOPIC")]
    pub ha_zigbee_lux_topic: String,
    #[arg(long, env = "XDG_RUNTIME_DIR")]
    pub runtime_dir: String,
}
