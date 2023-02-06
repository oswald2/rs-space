

use std::path::Path;

use tokio::io::{Error, ErrorKind, BufStream};
use tokio::fs::{read_to_string};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::tml_config::TMLConfig;
use crate::tml_message::TMLMessage;

pub async fn sle_connect(address: &str, cfg: &TMLConfig) -> Result<(), Error> {
    let mut sock = TcpStream::connect(address).await?;
    let mut stream = BufStream::new(sock);

    // we initiated the connection, so send a context message 
    let ctxt = TMLMessage::context_message(cfg.heartbeat, cfg.dead_factor);
    ctxt.write_to_async(&mut stream).await?;

    Ok(())
}
