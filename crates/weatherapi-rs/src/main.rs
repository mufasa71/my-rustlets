use clap::Parser;
use weatherapi_rs::{files::Cli, run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    if args.usage {
        Cli::generate_usage();
        return Ok(());
    }

    let res = run(args).await;

    if let Ok(e) = res {
        println!("{}", e);
    } else {
        println!("Error: {}", res.unwrap_err());
    }
    Ok(())
}
