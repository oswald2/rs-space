#[allow(unused)]
use std::time::Duration;

use rasn::types::{Utf8String, VisibleString};
use tokio::io::{BufStream, Error};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

use crate::asn1_raf::{ApplicationIdentifier, SlePdu, UnbindReason};
use crate::raf_config::RAFConfig;
use crate::raf_state::InternalRAFState;
// use crate::pdu::PDU;
use crate::tml_config::TMLConfig;
use crate::tml_message::TMLMessage;
use crate::types::sle::{string_to_service_instance_id, Credentials, SleVersion};
use log::{debug, error, info};

const QUEUE_SIZE: usize = 500;

pub enum SleMsg {
    Stop,
    PDU(SlePdu),
}

pub struct SleClientHandle {
    chan: Sender<SleMsg>,
    // we use an Option here, so that we can move out the JoinHandle from the struct for
    // awaiting on it
    thread: Option<JoinHandle<()>>,
}

impl SleClientHandle {
    /// Connect to the SLE RAF instance given in the RAFConfig.
    pub async fn sle_connect_raf(
        cfg: &TMLConfig,
        raf_config: &RAFConfig,
    ) -> Result<SleClientHandle, Error> {
        let address = format!("{}:{}", raf_config.hostname, raf_config.port);

        let sock = TcpStream::connect(address).await?;
        let mut stream = BufStream::new(sock);

        let (sender, mut receiver) = channel::<SleMsg>(QUEUE_SIZE);

        // we initiated the connection, so send a context message
        let ctxt = TMLMessage::context_message(cfg.heartbeat, cfg.dead_factor);
        ctxt.write_to_async(&mut stream).await?;

        let timeout = cfg.heartbeat;

        let hdl = tokio::spawn(async move {
            let mut raf_state = InternalRAFState::new();
            loop {
                select! {
                        res = receiver.recv() => {
                                match res {
                                    Some(SleMsg::Stop) => {
                                        debug!("Stop requested");
                                        return;
                                    }
                                    Some(SleMsg::PDU(pdu)) => {
                                        match process_sle_msg(pdu) {
                                            Err(err) => {
                                                error!("Error encoding SLE message: {}", err);
                                                break;
                                            }
                                            Ok(tml_message) => {
                                                // we got a valid TML message encoded, so send it
                                                if let Err(err) = tml_message.write_to_async(&mut stream).await {
                                                    error!("Error sending SLE message to socket: {}", err);
                                                    break;
                                                }
                                            }
                                        }
                                    },
                                    None => {
                                        debug!("Channel has been closed, returning...");
                                        break;
                                    }
                                }
                        },
                        res = TMLMessage::async_read(&mut stream) => {
                            match res {
                                Err(err) => {
                                    error!("Error reading SLE TML message from socket: {}", err);
                                    break;
                                }
                                Ok(msg) => {
                                    parse_sle_message(&msg, &mut raf_state);
                                }
                            }
                        },
                        _ = tokio::time::sleep(Duration::from_secs(timeout as u64)) => {
                            // we have a timeout, so send a heartbeat message
                            if let Err(err) = TMLMessage::heartbeat_message().write_to_async(&mut stream).await {
                                error!("Error sending SLE TML hearbeat message: {}", err);
                                break;
                            }
                        }
                }
            }
        });

        let ret = SleClientHandle {
            chan: sender,
            thread: Some(hdl),
        };

        Ok(ret)
    }

    /// Send a SleMsg as a command to control the machinery
    pub async fn command(&mut self, msg: SleMsg) -> Result<(), String> {
        if let Err(err) = self.chan.send(msg).await {
            return Err(format!("Could not process msg: {}", err));
        }
        Ok(())
    }

    /// Send a PDU to the connected instance
    pub async fn send_pdu(&mut self, pdu: SlePdu) -> Result<(), String> {
        self.command(SleMsg::PDU(pdu)).await
    }

    /// Bind the service given in the config to the end point, establish a connection and execute
    /// the SLE BIND operation
    pub async fn bind(&mut self, config: &RAFConfig) -> Result<(), String> {
        // first, convert the SII string from the config into a ASN1 structure
        let sii = string_to_service_instance_id(&config.sii)?;

        // Create the BIND SLE PDU
        let pdu = SlePdu::SleBindInvocation {
            // TODO: no credentials yet
            invoker_credentials: Credentials::Unused,
            initiator_identifier: VisibleString::new(Utf8String::from(&config.initiator)),
            responder_port_identifier: VisibleString::new(Utf8String::from(&config.responder_port)),
            service_type: (ApplicationIdentifier::RtnAllFrames as i32).into(),
            version_number: SleVersion::V3 as u16,
            service_instance_identifier: sii,
        };

        // And finally, send the PDU
        self.send_pdu(pdu).await
    }

    pub async fn unbind(&mut self, reason: UnbindReason) -> Result<(), String> {
        // TODO: no credentials yet
        let pdu = SlePdu::SleUnbindInvocation {
            invoker_credentials: Credentials::Unused,
            unbind_reason: (reason as i32).into(),
        };

        self.send_pdu(pdu).await
    }

    /// Stop the machinery
    pub async fn stop(&mut self) {
        let _ = self.command(SleMsg::Stop).await;
        if let Some(handle) = self.thread.take() {
            let _ = handle.await;
        }
    }
}

fn process_sle_msg(pdu: SlePdu) -> Result<TMLMessage, String> {
    match process_sle_pdu(pdu) {
        Err(err) => Err(format!("Error encoding PDU to ASN1: {}", err)),
        Ok(val) => Ok(TMLMessage::new_with_data(val)),
    }
}

fn process_sle_pdu(pdu: SlePdu) -> Result<Vec<u8>, rasn::ber::enc::Error> {
    rasn::der::encode(&pdu)
}

fn parse_sle_message(msg: &TMLMessage, state: &mut InternalRAFState) {
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    debug!("Decoded SLE PDU: {:?}", res);

    // TODO: credentials and authentication check

    match res {
        Ok(SlePdu::SleBindReturn {
            performer_credentials: _,
            responder_identifier,
            result,
        }) => {
            state.process_bind_return(&responder_identifier, result);
        }
        Ok(SlePdu::SleUnbindReturn {
            responder_credentials: _,
            result: _,
        }) => {
            state.process_unbind();
        }
        Ok(pdu) => {
            debug!("Received: {:?}", pdu);
        }
        Err(err) => {
            error!("Error on decoding SLE PDU: {err}");
        }
    }
}
