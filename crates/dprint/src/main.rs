use environment::{Environment, RealEnvironment};

#[macro_use]
mod types;

mod cli;
mod configuration;
mod environment;
mod format_context;
mod run_cli;
mod plugins;
mod utils;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> Result<(), types::ErrBox> {
    let environment = RealEnvironment::new();
    let plugin_resolver = plugins::wasm::WasmPluginResolver::new(&environment, &crate::plugins::wasm::compile);
    let args = std::env::args().collect();

    match run_cli::run_cli(args, &environment, &plugin_resolver).await {
        Err(err) => {
            environment.log_error(&format!("{:?}", err));
            std::process::exit(1);
        },
        _ => Ok(()),
    }
}

