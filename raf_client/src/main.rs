#[allow(unused)]

use std::fs::read_to_string;
use std::path::Path;

use rs_space_sle::user_config::UserConfig;
use tokio::io::{Error, ErrorKind};

use rustop::opts;

use log::{error, info, LevelFilter};
use log4rs::append::{console::ConsoleAppender, console::Target, file::FileAppender};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

mod application;

use crate::application::run_app;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (args, _rest) = opts! {
        synopsis "RAF SLE test client";
        opt config:Option<String>, desc: "Load config from the given file.";
    }
    .parse_or_exit();

    // initialise the logging
    let format_str = "{l} - {m}\n";
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(format_str)))
        .build("log/raf_client.log")?;
    let console = ConsoleAppender::builder().target(Target::Stderr).build();
    let log_config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(Appender::builder().build("stderr", Box::new(console)))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Debug),
        )
        .unwrap();
    match log4rs::init_config(log_config) {
        Err(err) => {
            let msg = format!("Error on initialising logger: {}", err);
            return Err(Error::new(ErrorKind::InvalidInput, msg));
        }
        Ok(_) => {}
    }

    // if specified, load the config
    let config: UserConfig = match args.config {
        Some(path) => match UserConfig::read_from_file(Path::new(&path)).await {
            Ok(cfg) => cfg,
            Err(err) => {
                error!("Error loading config from file {}: {}", path, err);
                UserConfig::default()
            }
        },
        None => UserConfig::default(),
    };

    // Now start the whole processing
    info!("RAF Client started");

    run_app(&config).await?;
    Ok(())
}
