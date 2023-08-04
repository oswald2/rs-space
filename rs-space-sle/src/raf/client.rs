use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
#[allow(unused)]
use std::time::Duration;

use rand::rngs::ThreadRng;
use rand::{thread_rng, RngCore};
use rasn::types::{Utf8String, VisibleString};
use rs_space_core::time::{Time, TimeEncoding};
use tokio::io::Error;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::asn1::*;
use crate::raf::asn1::{
    convert_frame, FrameOrNotification, RafTransferBuffer, RequestedFrameQuality,
};
use crate::raf::config::RAFConfig;
use crate::raf::state::{FrameCallback, InternalRAFState, RAFState};
use crate::sle::config::{CommonConfig, SleAuthType};
// use crate::pdu::PDU;
use crate::tml::config::TMLConfig;
use crate::tml::message::TMLMessage;
use crate::types::aul::{check_credentials, ISP1Credentials};
use crate::types::sle::{
    string_to_service_instance_id, to_conditional_ccsds_time, Credentials, PeerAbortDiagnostic,
};
use log::{debug, error, info, warn};

use super::asn1::RafStartReturnResult;

const QUEUE_SIZE: usize = 500;

pub enum SleMsg {
    Stop,
    PDU(SlePdu),
}

pub enum OpRet {
    BindRet(BindResult),
    UnbindRet,
    RafStartRet(RafStartReturnResult),
    AckRet(SleResult),
    PeerAbort,
}

type InternalTask = Option<JoinHandle<()>>;

/// The RAF client itself.
pub struct RAFClient {
    common_config: CommonConfig,
    raf_config: RAFConfig,
    chan: Option<Sender<SleMsg>>,
    ret_chan: Option<Receiver<OpRet>>,
    cancellation_token: CancellationToken,
    state: Arc<Mutex<InternalRAFState>>,
    // we use an Option here, so that we can move out the JoinHandle from the struct for
    // awaiting on it
    read_task: InternalTask,
    write_task: InternalTask,
    op_timeout: Duration,
    rand: ThreadRng,
    invoke_id: AtomicU16,
}

type InternalState = Arc<Mutex<InternalRAFState>>;

impl RAFClient {
    /// Create a new instance of a RAF client, with the given configurations and the given callback for
    /// TM Transfer Frames
    pub async fn new(
        common_config: &CommonConfig,
        raf_config: &RAFConfig,
        frame_callback: FrameCallback,
    ) -> Result<RAFClient, Error> {
        let cancellation = CancellationToken::new();
        let raf_state = Arc::new(Mutex::new(InternalRAFState::new(frame_callback)));

        let client = RAFClient {
            common_config: common_config.clone(),
            raf_config: raf_config.clone(),
            chan: None,
            ret_chan: None,
            cancellation_token: cancellation,
            state: raf_state,
            read_task: None,
            write_task: None,
            op_timeout: Duration::from_secs(raf_config.sle_operation_timeout as u64),
            rand: thread_rng(),
            invoke_id: AtomicU16::new(0),
        };

        Ok(client)
    }

    /// Send a SleMsg as a command to control the machinery
    pub async fn command(&mut self, msg: SleMsg) -> Result<(), String> {
        match &self.chan {
            Some(chan) => {
                if let Err(err) = chan.send(msg).await {
                    return Err(format!("Could not process msg: {}", err));
                }
                Ok(())
            }
            None => Err("Trying to send operation while not connected".to_string()),
        }
    }

    /// Send a PDU to the connected instance
    pub async fn send_pdu(&mut self, pdu: SlePdu) -> Result<(), String> {
        self.command(SleMsg::PDU(pdu)).await
    }

    fn new_credentials(&mut self) -> Credentials {
        let credentials = if self.common_config.auth_type == SleAuthType::AuthNone {
            Credentials::Unused
        } else {
            let isp1 = ISP1Credentials::new(
                self.common_config.hash_to_use,
                &Time::now(TimeEncoding::CDS8),
                self.rand.next_u32() as i32,
                &self.common_config.authority_identifier,
                &self.common_config.password,
            );
            Credentials::Used(isp1)
        };
        credentials
    }

    /// Bind the service given in the config to the end point, establish a connection and execute
    /// the SLE BIND operation
    pub async fn bind(&mut self) -> Result<(), String> {
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

        // Initiate the connection and start the read and write tasks
        let sock =
            match TcpStream::connect((self.raf_config.hostname.as_ref(), self.raf_config.port))
                .await
            {
                Ok(sock) => sock,
                Err(err) => {
                    return Err(format!("BIND error: could not connect to peer: {:?}", err));
                }
            };

        let (mut rx, mut tx) = sock.into_split();

        let (sender, mut receiver) = channel::<SleMsg>(QUEUE_SIZE);
        let sender2 = sender.clone();

        let (op_ret_sender, op_ret_receiver) = channel::<OpRet>(QUEUE_SIZE);
        let op_ret_sender2 = op_ret_sender.clone();

        let cancel1 = self.cancellation_token.clone();
        let cancel2 = self.cancellation_token.clone();

        let cfg: &TMLConfig = &self.common_config.tml;
        let timeout = cfg.heartbeat;
        let timeout2 = timeout.clone();
        let dead_factor2 = cfg.dead_factor.clone();
        let recv_timeout = cfg.heartbeat * cfg.dead_factor;
        let sii = self.raf_config.sii.clone();
        let sii2 = self.raf_config.sii.clone();

        let raf_state2 = self.state.clone();
        let raf_state3 = self.state.clone();

        let common_config2 = self.common_config.clone();
        let raf_config2 = self.raf_config.clone();

        let read_task = tokio::spawn(async move {
            loop {
                select!(
                    res = TMLMessage::async_read(&mut rx) => {
                        match res {
                            Err(err) => {
                                error!("Error reading SLE TML message from socket: {}", err);
                                break;
                            }
                            Ok(msg) => {
                                if msg.is_heartbeat() {
                                    debug!("SLE TML heartbeat received");
                                }
                                else
                                {
                                    parse_sle_message(&common_config2, &raf_config2, &msg, raf_state2.clone(), cancel1.clone(), &op_ret_sender).await;
                                }
                            }
                        }
                    },
                    _ = tokio::time::sleep(Duration::from_secs(recv_timeout as u64)) => {
                        // we have a receive heartbeat timeout, so report the error and disconnect
                        error!("Heartbeat timeout on service instance {}, terminating connection", sii);
                        let _ = sender2.send(SleMsg::Stop).await;
                        return;
                    }
                    _ = cancel1.cancelled() => {
                        debug!("RAF client for {} has been cancelled (read task)", sii);
                        return;
                    }
                );
            }
        });

        let write_task = tokio::spawn(async move {
            // we initiated the connection, so send a context message
            let ctxt = TMLMessage::context_message(timeout2, dead_factor2);
            if let Err(err) = ctxt.write_to_async(&mut tx).await {
                error!("Error sending SLE TML context message to provider: {err}");
                return;
            }

            loop {
                select! {
                    res = receiver.recv() => {
                                match res {
                                    Some(SleMsg::Stop) => {
                                        debug!("Stop requested");
                                        return;
                                    }
                                    Some(SleMsg::PDU(pdu)) => {
                                        debug!("Received command: {:?}", pdu);

                                        let is_peer_abort = pdu.is_peer_abort();

                                        match process_sle_msg(pdu, raf_state3.clone()) {
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

                                                // check, if we just sent a peer abort and if so, notify the client
                                                if is_peer_abort {
                                                    let _ = op_ret_sender2.send(OpRet::PeerAbort).await;
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
                        },
                        _ = cancel2.cancelled() => {
                            debug!("RAF client for {} has been cancelled (read task)", sii2);
                            if let Ok(tml) = process_sle_msg(SlePdu::SlePeerAbort{ diagnostic: PeerAbortDiagnostic::OtherReason}, raf_state3.clone()) {
                                let _ = tml.write_to_async(&mut tx).await;
                            }
                            return;
                        }

                }
            }
        });

        // update self with the tasks
        self.read_task = Some(read_task);
        self.write_task = Some(write_task);
        self.chan = Some(sender);
        self.ret_chan = Some(op_ret_receiver);

        // first, convert the SII string from the config into a ASN1 structure
        let sii = string_to_service_instance_id(&self.raf_config.sii)?;

        // generate the credentials
        let credentials = self.new_credentials();

        // Create the BIND SLE PDU
        let pdu = SlePdu::SleBindInvocation {
            invoker_credentials: credentials,
            initiator_identifier: self.common_config.authority_identifier.clone(),
            responder_port_identifier: VisibleString::new(Utf8String::from(
                &self.raf_config.responder_port,
            )),
            service_type: (ApplicationIdentifier::RtnAllFrames as i32).into(),
            version_number: self.raf_config.version as u16,
            service_instance_identifier: sii,
        };

        // And finally, send the PDU
        self.send_pdu(pdu).await?;

        // Now we wait for the operation return
        let chan: &mut Receiver<OpRet> = self.ret_chan.as_mut().unwrap();
        select! {
            ret = check_bind_return(chan) => {
                match ret {
                    Err(err) => {return Err(err); }
                    Ok(BindResult::BindOK(version)) => {
                        info!("BIND on {} successful with version {version}", self.raf_config.sii);
                    }
                    Ok(BindResult::BindDiag(diagnostic)) => {
                        return Err(format!("BIND error: {:?}", diagnostic));
                    }
                }
            }
            _ = tokio::time::sleep(self.op_timeout) => {
                return Err(format!("Error: timeout waiting for BIND RETURN"));
            }
            _ = self.cancellation_token.cancelled() => {
                debug!("RAF client for {} has been cancelled (BIND operation)", self.raf_config.sii);
            }
        }
        Ok(())
    }

    /// Unbind the client again from the endpoint
    pub async fn unbind(&mut self, reason: UnbindReason) -> Result<(), String> {
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

        // generate the credentials
        let credentials = self.new_credentials();

        let pdu = SlePdu::SleUnbindInvocation {
            invoker_credentials: credentials,
            unbind_reason: (reason as i32).into(),
        };

        self.send_pdu(pdu).await?;

        self.cancellation_token.cancel();

        // Take the tasks and wait for their termination
        if let Some(read_handle) = self.read_task.take() {
            let _ = read_handle.await;
        }
        if let Some(write_handle) = self.write_task.take() {
            let _ = write_handle.await;
        }

        // reset the client
        self.chan = None;
        self.ret_chan = None;
        self.invoke_id.store(0, Ordering::Relaxed);

        Ok(())
    }

    /// Start this service instance. The start- and stop time are provided 
    /// together with the requested frame quality
    pub async fn start(
        &mut self,
        start: Option<Time>,
        stop: Option<Time>,
        frame_quality: RequestedFrameQuality,
    ) -> Result<(), String> {
        // first check if we are in a correct state
        let state;
        {
            let st = self
                .state
                .lock()
                .expect("Error locking RAF internal state mutex");
            state = st.get_state();
        }
        if state == RAFState::Unbound || state == RAFState::Active {
            return Err("RAF START error: not in BOUND state".to_string());
        };

        // generate the credentials
        let credentials = self.new_credentials();

        let start_time = to_conditional_ccsds_time(start)?;
        let stop_time = to_conditional_ccsds_time(stop)?;

        // Create the RAF START SLE PDU
        let pdu = SlePdu::SleRafStartInvocation {
            invoker_credentials: credentials,
            invoke_id: self.invoke_id.fetch_add(1, Ordering::AcqRel),
            start_time: start_time,
            stop_time: stop_time,
            requested_frame_quality: (frame_quality as u32).into(),
        };

        // And finally, send the PDU
        self.send_pdu(pdu).await?;

        // Now we wait for the operation return
        let chan: &mut Receiver<OpRet> = self.ret_chan.as_mut().unwrap();
        select! {
            ret = check_raf_start_return(chan) => {
                match ret {
                    Err(err) => {return Err(err); }
                    Ok(RafStartReturnResult::PositiveResult) => {
                        info!("RAF START on {} successful", self.raf_config.sii);
                    }
                    Ok(RafStartReturnResult::NegativeResult(diagnostic)) => {
                        return Err(format!("RAF START error: {:?}", diagnostic));
                    }
                }
            }
            _ = tokio::time::sleep(self.op_timeout) => {
                return Err(format!("Error: timeout waiting for RAF START RETURN"));
            }
            _ = self.cancellation_token.cancelled() => {
                debug!("RAF client for {} has been cancelled (RAF START operation)", self.raf_config.sii);
            }
        }

        Ok(())
    }

    /// Stop the service instance again
    pub async fn stop(&mut self) -> Result<(), String> {
        // first check if we are in a correct state
        let state;
        {
            let st = self
                .state
                .lock()
                .expect("Error locking RAF internal state mutex");
            state = st.get_state();
        }
        if state != RAFState::Active {
            return Err("RAF STOP error: not in ACTIVE state".to_string());
        };

        // generate the credentials
        let credentials = self.new_credentials();

        // Create the RAF START SLE PDU
        let pdu = SlePdu::SleRafStopInvocation {
            invoker_credentials: credentials,
            invoke_id: self.invoke_id.fetch_add(1, Ordering::AcqRel),
        };

        // And finally, send the PDU
        self.send_pdu(pdu).await?;

        // Now we wait for the operation return
        let chan: &mut Receiver<OpRet> = self.ret_chan.as_mut().unwrap();
        select! {
            ret = check_raf_stop_return(chan) => {
                match ret {
                    Err(err) => {return Err(err); }
                    Ok(SleResult::PositiveResult) => {
                        info!("RAF STOP on {} successful", self.raf_config.sii);
                    }
                    Ok(SleResult::NegativeResult(diagnostic)) => {
                        return Err(format!("RAF STOP error: {:?}", diagnostic));
                    }
                }
            }
            _ = tokio::time::sleep(self.op_timeout) => {
                return Err(format!("Error: timeout waiting for RAF STOP RETURN"));
            }
            _ = self.cancellation_token.cancelled() => {
                debug!("RAF client for {} has been cancelled (RAF STOP operation)", self.raf_config.sii);
            }
        }

        Ok(())
    }

    /// Send a SLE PEER ABORT, then terminate all internal tasks
    pub async fn peer_abort(&mut self, diagnostic: PeerAbortDiagnostic) {
        warn!("Sending PeerAbort");
        // Create the RAF START SLE PDU
        let pdu = SlePdu::SlePeerAbort { diagnostic };

        // And finally, send the PDU
        let _ = self.send_pdu(pdu).await;

        // Now we wait for the operation return. In case of Peer Abort, we just
        // wait until it is sent, then we return
        let chan: &mut Receiver<OpRet> = self.ret_chan.as_mut().unwrap();
        let _ = check_peer_abort(chan).await;

        // Ok, we have sent the Peer Abort out, now we can terminate our own tasks
        self.cancel().await;
    }

    /// Sending the processing tasks a shutdown command
    pub async fn stop_processing(&mut self) {
        match &self.chan {
            Some(chan) => {
                let _ = chan.send(SleMsg::Stop).await;
            }
            None => {}
        }
        if let Some(handle) = self.write_task.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.read_task.take() {
            drop(handle);
        }
    }

    /// Cancel the internal tasks. 
    pub async fn cancel(&self) {
        self.cancellation_token.cancel();
    }
}

fn process_sle_msg(pdu: SlePdu, _state: InternalState) -> Result<TMLMessage, String> {
    match rasn::der::encode(&pdu) {
        Err(err) => Err(format!("Error encoding PDU to ASN1: {}", err)),
        Ok(val) => Ok(TMLMessage::new_with_data(val)),
    }
}

async fn parse_sle_message(
    config: &CommonConfig,
    raf_cfg: &RAFConfig,
    msg: &TMLMessage,
    state: InternalState,
    cancel_token: CancellationToken,
    op_ret_sender: &Sender<OpRet>,
) {
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    debug!("Decoded SLE PDU: {:?}", res);

    match res {
        Ok(SlePdu::SlePeerAbort { diagnostic: diag }) => {
            warn!("Received Peer Abort with diagnostic: {:?}", diag);
            cancel_token.cancel();
        }
        Ok(pdu) => {
            // check authentication
            if !check_authentication(config, raf_cfg, state.clone(), &pdu) {
                error!("SLE PDU failed authentication");
                cancel_token.cancel();
            }

            // then continue processing
            process_sle_pdu(&pdu, state, op_ret_sender).await;
        }
        Err(err) => {
            error!("Error on decoding SLE PDU: {err}");
        }
    }
}

async fn process_sle_pdu(pdu: &SlePdu, state: InternalState, op_ret_sender: &Sender<OpRet>) {
    match pdu {
        SlePdu::SleBindReturn {
            performer_credentials: _,
            responder_identifier,
            result,
        } => {
            // Lock our state and process the BIND RETURN
            {
                let mut lock = state.lock().expect("Mutex lock failed");
                lock.process_bind_return(&responder_identifier, result);
            }
            let _ = op_ret_sender.send(OpRet::BindRet(result.clone())).await;
        }
        SlePdu::SleUnbindReturn {
            responder_credentials: _,
            result: _,
        } => {
            {
                // Lock our state and process the UNBIND RETURN
                let mut lock = state.lock().expect("Mutex lock failed");
                lock.process_unbind();
            }

            let _ = op_ret_sender.send(OpRet::UnbindRet).await;
        }
        SlePdu::SleRafStartReturn {
            performer_credentials: _,
            invoke_id: _,
            result,
        } => {
            {
                let mut lock = state.lock().expect("Mutex lock failed");
                lock.process_start(result);
            }

            let _ = op_ret_sender.send(OpRet::RafStartRet(result.clone())).await;
        }
        SlePdu::SleAcknowledgement {
            credentials: _,
            invoke_id: _,
            result,
        } => {
            {
                let mut lock = state.lock().expect("Mutex lock failed");
                lock.process_stop(result);
            }
            let _ = op_ret_sender.send(OpRet::AckRet(result.clone())).await;
        }
        SlePdu::SleRafTransferBuffer(buffer) => {
            process_transfer_frame_buffer(state, buffer);
        }
        SlePdu::SlePeerAbort { diagnostic } => {
            let mut lock = state.lock().expect("Mutex lock failed");
            lock.process_peer_abort(diagnostic);
        }
        pdu => {
            debug!("Received: {:?}", pdu);
        }
    }
}

fn check_authentication(
    config: &CommonConfig,
    _raf_cfg: &RAFConfig,
    state: InternalState,
    pdu: &SlePdu,
) -> bool {
    match config.auth_type {
        SleAuthType::AuthNone =>
        // in case we have not authentication configured, all is good
        {
            true
        }
        SleAuthType::AuthBind =>
        // for AUTH_BIND, we need to only check the BIND and BIND RETURN invocations
        {
            match pdu {
                SlePdu::SleBindInvocation {
                    invoker_credentials,
                    initiator_identifier,
                    ..
                } => match invoker_credentials {
                    Credentials::Unused => {
                        error!("BIND Authentication failed: AUTH_BIND requested, but BIND invocation does not contain credentials");
                        return false;
                    }
                    Credentials::Used(creds) => {
                        check_peer(config, creds, initiator_identifier, "BIND")
                    }
                },
                SlePdu::SleBindReturn {
                    performer_credentials,
                    responder_identifier,
                    ..
                } => match performer_credentials {
                    Credentials::Unused => {
                        error!("BIND Authentication failed: AUTH_BIND requested, but BIND RETURN does not contain credentials");
                        return false;
                    }
                    Credentials::Used(isp1) => {
                        check_peer(config, isp1, responder_identifier, "BIND RETURN")
                    }
                },
                _ => {
                    // All other SLE PDUs are fine without authentication
                    return true;
                }
            }
        }
        SleAuthType::AuthAll => check_all(config, state, pdu),
    }
}

fn check_all(config: &CommonConfig, state: InternalState, pdu: &SlePdu) -> bool {
    let credentials = pdu.get_credentials();

    if let SlePdu::SleRafTransferBuffer(buffer) = pdu {
        for frame in buffer {
            let res = check_buffer_credential(config, state.clone(), frame);
            if !res {
                return false;
            }
        }
        return true;
    } else {
        match credentials {
            Some(creds) => match creds {
                Credentials::Unused => {
                    error!("Authentication failed, no credentials provided");
                    return false;
                }
                Credentials::Used(isp1) => {
                    let name;
                    {
                        let lock = state.lock().expect("Mutex lock failed");
                        name = lock.provider().clone();
                    }
                    let op_name = pdu.operation_name();
                    return check_peer(config, isp1, &name, op_name);
                }
            },
            None => {
                return true;
            }
        }
    }
}

fn check_buffer_credential(
    config: &CommonConfig,
    state: InternalState,
    buf_part: &FrameOrNotification,
) -> bool {
    let creds = match buf_part {
        FrameOrNotification::AnnotatedFrame(trans) => &trans.invoker_credentials,
        FrameOrNotification::SyncNotification(notif) => &notif.invoker_credentials,
    };

    match creds {
        Credentials::Unused => {
            error!("Authentication of TM frame failed, no credentials provided");
            return false;
        }
        Credentials::Used(isp1) => {
            let name;
            {
                let lock = state.lock().expect("Mutex lock failed");
                name = lock.provider().clone();
            }
            let op_name = "RAF TRANSFER BUFFER";
            return check_peer(config, &isp1, &name, op_name);
        }
    }
}

fn check_peer(
    config: &CommonConfig,
    isp1: &ISP1Credentials,
    identifier: &VisibleString,
    op_name: &str,
) -> bool {
    match config.get_peer(identifier) {
        Some(peer) => {
            if !check_credentials(&isp1, identifier, &peer.password) {
                error!("{}: authentication failed", op_name);
                return false;
            }
            return true;
        }
        None => {
            error!(
                "Peer '{}' is not configured, authentication failed!",
                identifier.value.as_str()
            );
            return false;
        }
    }
}

fn process_transfer_frame_buffer(state: InternalState, buffer: &RafTransferBuffer) {
    let lock = state.lock().expect("Mutex lock failed");

    for elem in buffer {
        match elem {
            FrameOrNotification::AnnotatedFrame(frame) => match convert_frame(frame) {
                Ok(frame) => {
                    lock.process_tm_frame(&frame);
                }
                Err(err) => {
                    error!("Error decoding TM frame: {}", err);
                }
            },
            FrameOrNotification::SyncNotification(notif) => {
                debug!("Got SYNC Notification: {:?}", notif);
            }
        }
    }
}

async fn check_bind_return(chan: &mut Receiver<OpRet>) -> Result<BindResult, String> {
    loop {
        match chan.recv().await {
            None => {
                return Err(format!(
                    "Error: internal operation return channel has been closed"
                ));
            }
            Some(OpRet::BindRet(res)) => {
                return Ok(res);
            }
            Some(_) => {}
        }
    }
}

async fn check_raf_start_return(
    chan: &mut Receiver<OpRet>,
) -> Result<RafStartReturnResult, String> {
    loop {
        match chan.recv().await {
            None => {
                return Err(format!(
                    "Error: internal operation return channel has been closed"
                ));
            }
            Some(OpRet::RafStartRet(res)) => {
                return Ok(res);
            }
            Some(_) => {}
        }
    }
}

async fn check_raf_stop_return(chan: &mut Receiver<OpRet>) -> Result<SleResult, String> {
    loop {
        match chan.recv().await {
            None => {
                return Err(format!(
                    "Error: internal operation return channel has been closed"
                ));
            }
            Some(OpRet::AckRet(res)) => {
                return Ok(res);
            }
            Some(_) => {}
        }
    }
}

async fn check_peer_abort(chan: &mut Receiver<OpRet>) -> Result<(), String> {
    loop {
        match chan.recv().await {
            None => {
                return Err(format!(
                    "Error: internal operation return channel has been closed"
                ));
            }
            Some(OpRet::PeerAbort) => {
                return Ok(());
            }
            Some(_) => {}
        }
    }
}
