use std::path::PathBuf;

use aicommit_rs::{
    commit::{generate_commit, read_template},
    config::get_config,
    diff::get_diff,
};
use clap::{Command, ValueHint, arg, crate_version, value_parser};

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

    let config = get_config();
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
