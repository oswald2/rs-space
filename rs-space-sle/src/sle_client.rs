#[allow(unused)]
use std::time::Duration;

use tokio::io::{BufStream, Error};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

use crate::asn1_raf::{SlePdu};
use crate::raf_config::RAFConfig;
// use crate::pdu::PDU;
use crate::tml_config::TMLConfig;
use crate::tml_message::TMLMessage;
use log::{error, info};

const QUEUE_SIZE: usize = 500;

pub struct SleMsg(SlePdu);

pub struct SleClientHandle {
    chan: Sender<SleMsg>,
    thread: JoinHandle<()>
}

impl SleClientHandle {
    pub async fn send_pdu(&mut self, pdu: SlePdu) -> Result<(), String> {
        if let Err(err) = self.chan.send(SleMsg(pdu)).await {
            return Err(format!("Could not send PDU: {}", err));
        }
        Ok(())
    }
}

pub async fn sle_connect_raf(cfg: &TMLConfig, raf_config: &RAFConfig) -> Result<SleClientHandle, Error> {
    let address = format!("{}:{}", raf_config.hostname, raf_config.port);

    let sock = TcpStream::connect(address).await?;
    let mut stream = BufStream::new(sock);

    let (sender, mut receiver) = channel::<SleMsg>(QUEUE_SIZE);

    // we initiated the connection, so send a context message
    let ctxt = TMLMessage::context_message(cfg.heartbeat, cfg.dead_factor);
    ctxt.write_to_async(&mut stream).await?;

    let timeout = cfg.heartbeat;

    let hdl = tokio::spawn(async move {
        loop {
            select! {
                    res = receiver.recv() => {
                            match res {
                                Some(msg) => {
                                    match process_sle_msg(msg) {
                                        Err(err) => {
                                            error!("Error encoding SLE message: {}", err);
                                            break;
                                        }
                                        Ok(tml_message) => {
                                            if let Err(err) = tml_message.write_to_async(&mut stream).await {
                                                error!("Error sending SLE message to socket: {}", err);
                                                break;
                                            }
                                        }
                                    }
                                },
                                None => {
                                    break;
                                }
                            }
                    },
                    res = TMLMessage::async_read(&mut stream) => {
                        match res {
                            Err(err) => error!("Error reading SLE TML message from socket: {}", err),
                            Ok(msg) => {
                                info!("Received TML Message: {:?}", msg);
                                parse_sle_message(&msg);
                            }
                        }
                    },
                    _ = tokio::time::sleep(Duration::from_secs(timeout as u64)) => {
                        // we have a timeout, so send a heartbeat message
                        if let Err(err) = TMLMessage::heartbeat_message().write_to_async(&mut stream).await {
                            error!("Error sending SLE TML hearbeat message: {}", err);
                        }
                    }
            }
        }
    });

    let ret = SleClientHandle { chan: sender, thread: hdl };

    Ok(ret)
}

fn process_sle_msg(msg: SleMsg) -> Result<TMLMessage, String> {
    match msg {
        SleMsg(pdu) => match process_sle_pdu(pdu) {
            Err(err) => Err(format!("Error encoding PDU to ASN1: {}", err)),
            Ok(val) => Ok(TMLMessage::new_with_data(val)),
        },
    }
}

fn process_sle_pdu(pdu: SlePdu) -> Result<Vec<u8>, rasn::ber::enc::Error> {
    rasn::der::encode(&pdu)
}

fn parse_sle_message(msg: &TMLMessage) {
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    info!("Received: {:?}", res);
}
