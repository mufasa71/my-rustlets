use std::path::PathBuf;

use aicommit_rs::{
    commit::{generate_commit, read_template},
    config::Config,
    diff::get_diff,
};
use clap::{Arg, Command, ValueHint, arg, crate_version, value_parser};

fn build_cli() -> Command {
    let mut template_path = dirs::home_dir().expect("home dir expected");
    template_path.push(".aicommit-template");

    Command::new("aicommit-rs")
        .version(crate_version!())
        .about("Uses OpenAI or Google AI to generate commit message suggestions based on the diff between the current branch and master.
Then, you can select a commit message from the list and use it to commit your changes.")
        .next_line_help(true)
        .arg(
            arg!(-t --template <FILE> "Specify a custom template")
                .value_hint(ValueHint::AnyPath)
                .default_value(template_path.into_os_string())
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("api-key")
                .long("api-key")
                .value_name("API_KEY")
                .help("Specify OpenAI API key")
                .required(true)
                .env("AI_COMMIT_API_KEY"),
        )
        .arg(
            Arg::new("api-url")
                .long("api-url")
                .value_name("API_URL")
                .help("Specify OpenAI API endpoint")
                .required(true),
        )
        .arg(arg!(--model <MODEL_NAME> "Specify model name").required(true))
        .arg(arg!(--usage "Show usage").required(false))
}

#[tokio::main]
async fn main() {
    let matches = build_cli().get_matches();

    if matches.get_flag("usage") {
        let mut cmd = build_cli();
        eprintln!("Generating usage spec...");
        clap_usage::generate(&mut cmd, "aicommit-rs", &mut std::io::stdout());
        return;
    }

    let config = Config {
        openai_api_key: matches
            .get_one::<String>("api-key")
            .expect("API key is required")
            .to_string(),
        openai_api_url: matches
            .get_one::<String>("api-url")
            .expect("API URL is required")
            .to_string(),
        model_name: matches
            .get_one::<String>("model")
            .expect("Model name is required")
            .to_string(),
    };
    let diff = get_diff().expect("Error getting diff");
    let template = read_template(
        matches
            .get_one::<PathBuf>("template")
            .expect("No default template provided"),
    )
    .expect("Failed to read template");

    let result = generate_commit(template.replace("{{diff}}", &diff), config)
        .await
        .expect("Error generating commit");

    println!("{}", result);
}

#[test]
fn verify_cmd() {
    build_cli().debug_assert();
}
