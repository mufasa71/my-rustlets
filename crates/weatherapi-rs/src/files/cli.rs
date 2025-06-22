use std::fs::File;

use clap::{crate_version, Args, Command, Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Format {
    Json,
    Compact,
}

#[derive(Parser, Debug)]
#[command(version = crate_version!(), about = "Weather CLI tool", long_about = None)]
pub struct Cli {
    #[arg(
        short,
        required_unless_present("usage"),
        long_help = "City name e.g. London, Paris"
    )]
    pub q: Option<String>,
    #[arg(value_enum, short, long, default_value_t = Format::Compact, long_help = "Output format")]
    pub format: Format,
    #[arg(long, long_help = "Include air quality index")]
    pub aqi: bool,
    #[arg(
        long,
        long_help = "Generate usage file in KDL format (usefull to create auto-completion)"
    )]
    pub usage: bool,
}

impl Cli {
    pub fn generate_usage() {
        let cmd = Command::new("weatherapi-rs");
        let mut cmd = Cli::augment_args(cmd);

        if cfg!(debug_assertions) {
            let mut file = File::create("docs/usage.spec.kdl").unwrap();
            clap_usage::generate(&mut cmd, "weatherapi-rs", &mut file);
        } else {
            clap_usage::generate(&mut cmd, "weatherapi-rs", &mut std::io::stdout());
        }
    }
}

#[test]
fn verify_cmd() {
    let cmd = Command::new("weatherapi-rs");
    let cmd = Cli::augment_args(cmd);

    cmd.debug_assert();
}
