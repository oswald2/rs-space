use std::sync::{Arc, Mutex};
#[allow(unused)]
use std::time::Duration;

use rasn::types::{Utf8String, VisibleString};
use tokio::io::Error;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

use crate::asn1_raf::{ApplicationIdentifier, SlePdu, UnbindReason};
use crate::raf::config::RAFConfig;
use crate::raf::state::{InternalRAFState, RAFState};
// use crate::pdu::PDU;
use crate::tml::config::TMLConfig;
use crate::tml::message::TMLMessage;
use crate::types::sle::{string_to_service_instance_id, Credentials, SleVersion};
use log::{debug, error, info};

use function_name::named;

const QUEUE_SIZE: usize = 500;

pub enum SleMsg {
    Stop,
    PDU(SlePdu),
}

pub struct SleClientHandle {
    chan: Sender<SleMsg>,
    state: Arc<Mutex<InternalRAFState>>,
    // we use an Option here, so that we can move out the JoinHandle from the struct for
    // awaiting on it
    read_task: Option<JoinHandle<()>>,
    write_task: Option<JoinHandle<()>>,
}

type InternalState = Arc<Mutex<InternalRAFState>>;

impl SleClientHandle {
    /// Connect to the SLE RAF instance given in the RAFConfig.
    pub async fn sle_connect_raf(
        cfg: &TMLConfig,
        raf_config: &RAFConfig,
    ) -> Result<SleClientHandle, Error> {
        let sock = TcpStream::connect((raf_config.hostname.as_ref(), raf_config.port)).await?;

        let (mut rx, mut tx) = sock.into_split();

        let (sender, mut receiver) = channel::<SleMsg>(QUEUE_SIZE);

        let timeout = cfg.heartbeat;

        let raf_state = Arc::new(Mutex::new(InternalRAFState::new()));
        let raf_state2 = raf_state.clone();
        let raf_state3 = raf_state.clone();

        let hdl1 = tokio::spawn(async move {
            loop {
                match TMLMessage::async_read(&mut rx).await {
                    Err(err) => {
                        error!("Error reading SLE TML message from socket: {}", err);
                        break;
                    }
                    Ok(msg) => {
                        parse_sle_message(&msg, raf_state.clone());
                    }
                }
            }
        });

        // we initiated the connection, so send a context message
        let ctxt = TMLMessage::context_message(cfg.heartbeat, cfg.dead_factor);
        ctxt.write_to_async(&mut tx).await?;

        let hdl2 = tokio::spawn(async move {
            loop {
                select! {
                    res = receiver.recv() => {
                                match res {
                                    Some(SleMsg::Stop) => {
                                        debug!("Stop requested");
                                        return;
                                    }
                                    Some(SleMsg::PDU(pdu)) => {
                                        match process_sle_msg(pdu, raf_state2.clone()) {
                                            Err(err) => {
                                                error!("Error encoding SLE message: {}", err);
                                                break;
                                            }
                                            Ok(tml_message) => {
                                                // we got a valid TML message encoded, so send it
                                                if let Err(err) = tml_message.write_to_async(&mut tx).await {
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

                        _ = tokio::time::sleep(Duration::from_secs(timeout as u64)) => {
                            // we have a timeout, so send a heartbeat message
                            if let Err(err) = TMLMessage::heartbeat_message().write_to_async(&mut tx).await {
                                error!("Error sending SLE TML hearbeat message: {}", err);
                                break;
                            }
                        }
                }
            }
        });

        let ret = SleClientHandle {
            chan: sender,
            state: raf_state3,
            read_task: Some(hdl1),
            write_task: Some(hdl2),
        };

        Ok(ret)
    }

    /// Send a SleMsg as a command to control the machinery
    #[named]
    pub async fn command(&mut self, msg: SleMsg) -> Result<(), String> {
        debug!(function_name!());
        if let Err(err) = self.chan.send(msg).await {
            return Err(format!("Could not process msg: {}", err));
        }
        Ok(())
    }

    /// Send a PDU to the connected instance
    #[named]
    pub async fn send_pdu(&mut self, pdu: SlePdu) -> Result<(), String> {
        debug!(function_name!());
        self.command(SleMsg::PDU(pdu)).await
    }

    /// Bind the service given in the config to the end point, establish a connection and execute
    /// the SLE BIND operation
    #[named]
    pub async fn bind(&mut self, config: &RAFConfig) -> Result<(), String> {
        debug!(function_name!());

        // first check if we are in a correct state
        let state;
        {
            let st = self
                .state
                .lock()
                .expect("Error locking RAF internal state mutex");
            state = st.get_state();
        }
        if state == RAFState::Bound || state == RAFState::Active {
            return Err("BIND error: not in UNBOUND state".to_string());
        }

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

    #[named]
    pub async fn unbind(&mut self, reason: UnbindReason) -> Result<(), String> {
        debug!(function_name!());

        // first check if we are in a correct state
        let state;
        {
            let st = self
                .state
                .lock()
                .expect("Error locking RAF internal state mutex");
            state = st.get_state();
        }
        if state == RAFState::Unbound {
            // Unbind is always accepted, though in UNBOUND state, we don't need
            // to send
            return Ok(());
        }

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
        if let Some(handle) = self.write_task.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.read_task.take() {
            drop(handle);
        }
    }
}

fn process_sle_msg(pdu: SlePdu, state: InternalState) -> Result<TMLMessage, String> {
    match process_sle_pdu(pdu) {
        Err(err) => Err(format!("Error encoding PDU to ASN1: {}", err)),
        Ok(val) => Ok(TMLMessage::new_with_data(val)),
    }
}

fn process_sle_pdu(pdu: SlePdu) -> Result<Vec<u8>, rasn::ber::enc::Error> {
    rasn::der::encode(&pdu)
}

fn parse_sle_message(msg: &TMLMessage, state: InternalState) {
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    debug!("Decoded SLE PDU: {:?}", res);

    // TODO: credentials and authentication check

    match res {
        Ok(SlePdu::SleBindReturn {
            performer_credentials: _,
            responder_identifier,
            result,
        }) => {
            let mut lock = state.lock().expect("Mutex lock failed");
            lock.process_bind_return(&responder_identifier, result);
        }
        Ok(SlePdu::SleUnbindReturn {
            responder_credentials: _,
            result: _,
        }) => {
            let mut lock = state.lock().expect("Mutex lock failed");
            lock.process_unbind();
        }
        Ok(pdu) => {
            debug!("Received: {:?}", pdu);
        }
        Err(err) => {
            error!("Error on decoding SLE PDU: {err}");
        }
    }
}
