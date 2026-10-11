use clap::Parser;
use mqtt_to_dbus::cli::Cli;

#[derive(Parser)]
pub struct LuxCli {
    #[command(flatten)]
    pub mqtt: Cli,
    #[arg(long, env = "XDG_RUNTIME_DIR")]
    pub runtime_dir: String,
    // observed max raw illuminance for this sensor
    #[arg(long)]
    pub max_illuminance: f64,
    // dead zone between toggle to avoid sending the same command repeatedly
    #[arg(long, default_value_t = 0.35)]
    pub light_threshold: f64,
    #[arg(long, default_value_t = 0.30)]
    pub dark_threshold: f64,
}
