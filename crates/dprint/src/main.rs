use environment::{Environment, RealEnvironment};

#[macro_use]
mod types;

mod configuration;
mod environment;
mod run_cli;
mod plugins;

#[tokio::main]
async fn main() -> Result<(), types::ErrBox> {
    let environment = RealEnvironment::new();
    let args = std::env::args().collect();

    match run_cli::run_cli(&environment, args).await {
        Err(err) => {
            environment.log_error(&format!("{:?}", err));
            std::process::exit(1);
        },
        _ => Ok(()),
    }
}

