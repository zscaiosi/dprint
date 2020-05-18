use environment::{Environment, RealEnvironment};

#[macro_use]
mod types;
mod environment;

mod cli;
mod configuration;
mod plugins;
mod utils;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> Result<(), types::ErrBox> {
    let args = cli::parse_args(std::env::args().collect())?;
    let environment = RealEnvironment::new(args.verbose);
    let plugin_resolver = plugins::wasm::WasmPluginResolver::new(&environment, &crate::plugins::wasm::compile);

    match cli::run_cli(args, &environment, &plugin_resolver).await {
        Err(err) => {
            environment.log_error(&err.to_string());
            std::process::exit(1);
        },
        _ => Ok(()),
    }
}

