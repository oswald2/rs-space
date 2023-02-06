

use tokio::io::{Error, ErrorKind, BufStream};

use log::{info, error};

use rs_space_sle::tml_config::TMLConfig;
use rs_space_sle::sle_client::{sle_connect};

const DEFAULT_CONFIG: TMLConfig = TMLConfig {    
    heartbeat: 30,
    dead_factor: 2,
    server_init_time: 30,
    min_heartbeat: 3,
    max_heartbeat: 3600,
    min_dead_factor: 2,
    max_dead_factor: 60,
};


pub async fn run_app(address: String) -> Result<(), Error> {
    info!("Connecting to {}...", address);

    sle_connect(&address, &DEFAULT_CONFIG).await?;

    std::thread::sleep(std::time::Duration::from_secs(5));
    Ok(())
}
