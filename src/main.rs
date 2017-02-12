#[macro_use]
extern crate prometheus;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate dotenv;
extern crate rusoto;
extern crate ctrlc;

mod config;
mod poller;
mod periodic;
mod server;
mod termination;

use std::time::Duration;
use hyper::server::Server;
use config::{ExporterConfigurationProvider, AwsSettingsProvider};
use server::DeucalionHandler;
use poller::AwsPoller;
use periodic::AsyncPeriodicRunner;
use termination::TerminationGuard;
use prometheus::TextEncoder;

fn inject_environment() {
    match dotenv::dotenv() {
        Ok(_) | Err(dotenv::DotenvError::Io) => // it is ok if the .env file was not found
            return,
        Err(dotenv::DotenvError::Parsing {line}) =>
            panic!(".env file parsing failed at {:?}", line),
        Err(err) => panic!(err)
    }
}

fn main() {
    inject_environment();

    let config = config::EnvironmentSettingsProvider::new().unwrap();
    let poller = AwsPoller::new(&config).unwrap();
    let polling_period = config.polling_period().unwrap_or(Duration::from_secs(5));

    println!("listening address {:?} {:?} {:?}", config.listen_on(),
        config.read_timeout(), config.keep_alive_timeout());
    let mut listening = Server::http(config.listen_on())
        .unwrap()
        .handle(DeucalionHandler::new(TextEncoder::new()))
        .unwrap();
    let _runner = AsyncPeriodicRunner::new(poller, polling_period);
    TerminationGuard::new();

    let _ = listening.close();
}