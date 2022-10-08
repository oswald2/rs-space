use std::path::Path;

use tokio::io::{Error, ErrorKind};

use tokio::fs::{read_to_string};

use tokio::net::TcpStream;

use tokio::sync::mpsc;

use log::{info, error};

use async_recursion::async_recursion;

use rs_space_core::edsl::{EDSL, Action};
use rs_space_core::edsl::Action::{SendPkt, RepeatN, Log};
use rs_space_core::ccsds_packet::{FastCcsdsPacket};


pub async fn run_app(script_file: &Path, address: String) -> Result<(), Error> {
    // First spawn the sender thread 
    let (tx, rx) = mpsc::channel(500);

    tokio::spawn(async move {
        send_thread(address, rx).await;
    });

    // Open the file and parse it 
    let content = read_to_string(script_file).await?;

    let actions: EDSL = serde_json::from_str(&content)?;

    edsl_processor(&tx, actions).await
}


async fn send_thread(address: String, mut chan: mpsc::Receiver<FastCcsdsPacket>) {
    // Connect to the specified socket
    match TcpStream::connect(address).await {
        Ok(mut stream) => {
            // receive data from the channel and send them to the 
            // socket 
            while let Some(pkt) = chan.recv().await {
                match pkt.write_to_async(&mut stream).await {
                    Ok(_) => {},
                    Err(err) => {
                        error!("Could not write to socket: {}", err)
                    }
                    
                }
            }
        },
        Err(err) => { error!("Error connecting: {}", err); }
    }
}


async fn edsl_processor(tx: &mpsc::Sender<FastCcsdsPacket>, edsl: EDSL) -> Result<(), Error> {
    for action in &edsl.actions {
        action_processor(tx, action).await?
    }
    Ok(())
}

#[async_recursion]
async fn action_processor(tx: &mpsc::Sender<FastCcsdsPacket>, actions: &Action) -> Result<(), Error> {
    match actions {
        SendPkt(pkt) => {
            let ccsds_pkt = pkt.to_ccsds_packet()?;  
            let fast_pkt = ccsds_pkt.to_fast_ccsds_pkt();

            match tx.send(fast_pkt).await {
                Ok(()) => {}, 
                Err(err) => {
                    error!("Error on sending packet to channel: {}", err);
                    return Err(Error::new(ErrorKind::Other, "could not send packet to channel"));
                }
            }
        },
        RepeatN(n, actions) => {
            for _ in [0..*n] {
                action_processor(tx, actions).await?
            }
        }
        Log(str) => {
            info!("{}", str);
        }
    }
    
    Ok(())
}