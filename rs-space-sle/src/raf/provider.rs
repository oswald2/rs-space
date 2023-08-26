use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use core::sync::atomic::Ordering;
use std::sync::atomic::AtomicI32;

use log::{debug, error, info, warn};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rasn::types::*;

use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    select,
    sync::{
        mpsc::{channel, Receiver, Sender},
        watch,
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::{
    asn1::*,
    provider::raf_interface::ProviderNotifier,
    raf::asn1::*,
    sle::config::{CommonConfig, SleAuthType},
    tml::message::TMLMessage,
    types::{
        aul::{check_credentials, ISP1Credentials},
        sle::*,
    },
};
use rs_space_core::{
    time::{Time, TimeEncoding},
    timed_buffer::{self, TimedBuffer},
};

use super::{
    asn1::FrameOrNotification,
    config::RAFProviderConfig,
    provider_state::InternalRAFProviderState,
    state::{AtomicRAFState, RAFState},
};

const QUEUE_SIZE: usize = 500;

pub enum SleMsg {
    Stop,
    BindReturn(SlePdu, u16, u16),
    PDU(SlePdu),
}

type InternalState = Arc<Mutex<InternalRAFProviderState>>;

type Notifier = Box<dyn ProviderNotifier + Send>;

pub enum DataBufferElement {
    Frame(SleFrame),
    Notification(Notification),
}

pub struct RAFProvider {
    common_config: CommonConfig,
    raf_config: RAFProviderConfig,
    state: InternalState,
    cancel_token: CancellationToken,
    raf_state: Arc<AtomicRAFState>,
    state_watch: tokio::sync::watch::Receiver<RAFState>,
    state_watch_snd: Option<tokio::sync::watch::Sender<RAFState>>,
    chan: Option<Sender<SleMsg>>,
    buffer_sender: Option<timed_buffer::Sender<DataBufferElement>>,
    read_handle: Option<JoinHandle<()>>,
    write_handle: Option<JoinHandle<()>>,
}

struct Args {
    common_config: CommonConfig,
    raf_config: RAFProviderConfig,
    state: InternalState,
    cancel_token: CancellationToken,
    app_notifier: Notifier,
    chan: Sender<SleMsg>,
    rand: StdRng,
    rx: OwnedReadHalf,
    interval: u16,
    dead_factor: u16,
    state_watch: tokio::sync::watch::Sender<RAFState>,
}

impl RAFProvider {
    pub fn new(common_config: &CommonConfig, raf_config: &RAFProviderConfig) -> RAFProvider {
        let raf_state = Arc::new(AtomicRAFState::new(RAFState::Unbound));
        let (state_tx, state_rx) = watch::channel(RAFState::Unbound);

        RAFProvider {
            common_config: common_config.clone(),
            raf_config: raf_config.clone(),
            state: Arc::new(Mutex::new(InternalRAFProviderState::new(
                common_config,
                raf_config,
                raf_state.clone(),
            ))),
            cancel_token: CancellationToken::new(),
            raf_state: raf_state,
            state_watch: state_rx,
            state_watch_snd: Some(state_tx),
            chan: None,
            buffer_sender: None,
            read_handle: None,
            write_handle: None,
        }
    }

    pub async fn run(
        &mut self,
        notifier: Box<dyn ProviderNotifier + Send>,
    ) -> tokio::io::Result<()> {
        let listener =
            TcpListener::bind((self.raf_config.hostname.as_ref(), self.raf_config.port)).await?;

        let (socket, peer) = listener.accept().await?;

        info!(
            "Connection on RAF instance {} from {}",
            self.raf_config.sii, peer
        );

        // a mpsc channel to send command messags to the writer task.
        let (sender, mut receiver) = channel::<SleMsg>(QUEUE_SIZE);
        let sender2 = sender.clone();
        let sender3 = sender.clone();

        // We need a writer and a reader task, so we split the socket into a
        // read and write half
        let (rx, tx) = socket.into_split();

        // Create the timed buffer channel
        let (buf_sender, buf_receiver) = TimedBuffer::<DataBufferElement>::new(
            self.raf_config.buffer_size as usize,
            Duration::from_millis(self.raf_config.latency as u64),
        );

        self.chan = Some(sender);
        self.buffer_sender = Some(buf_sender);

        // some clones to pass on to the two tasks
        let cancel2 = self.cancel_token.clone();
        let cancel3 = self.cancel_token.clone();
        let cancel4 = self.cancel_token.clone();

        let config2 = self.common_config.clone();
        let config3 = self.common_config.clone();
        let config4 = self.common_config.clone();

        let raf_config2 = self.raf_config.clone();
        let raf_config3 = self.raf_config.clone();
        let raf_config4 = self.raf_config.clone();

        let state2 = self.state.clone();

        //let notifier = self.app_notifier.clone();

        // The server timeout is for waiting for the TML Context message to be received
        let server_timeout = Duration::from_secs(self.raf_config.server_init_time as u64);

        // Remove the watch sender from the provider and move it into
        // the reader thread, so that it can notify the provider
        let state_watch = self.state_watch_snd.take().unwrap();

        // The first task. This task reads from the socket, parses the SLE PDUs,
        // authenticates them (if configured) and passes them on forwards to the
        // application. Also, PDUs for the state machine are processed, forwarded
        // to the state and a subsequent return operation is generated and passed
        // to the writer task
        let read_handle = tokio::spawn(async move {
            let interval = config2.tml.heartbeat;
            let dead_factor = config2.tml.dead_factor;
            let cancel_clone = cancel2.clone();

            let mut args = Args {
                common_config: config2,
                raf_config: raf_config2,
                state: state2,
                cancel_token: cancel2,
                app_notifier: notifier,
                chan: sender2,
                rand: SeedableRng::from_entropy(),
                rx,
                interval,
                dead_factor,
                state_watch,
            };

            // First, we expect a TML context message. If not, we bail out
            match read_context_message(&mut args, server_timeout).await {
                Err(err) => {
                    error!("Error reading SLE TML Context Message: {err}");
                    cancel_clone.cancel();
                }
                Ok((interval, dead_factor)) => {
                    args.interval = interval;
                    args.dead_factor = dead_factor;

                    // Do the actual work. This function loops and processes the incoming PDUs
                    match read_pdus(&mut args).await {
                        Err(err) => {
                            error!("{err}");
                            cancel_clone.cancel();
                        }
                        Ok(_) => {
                            cancel_clone.cancel();
                        }
                    }
                }
            }
        });

        // The writer task. This listens on the mpsc channel for messages and reacts to them.
        // The primary task is of course getting SLE PDUs via this channel, add
        // authentication info if configured, encode them and send them to the socket.
        let write_handle = tokio::spawn(async move {
            match write_task(&config3, &raf_config3, tx, &mut receiver, cancel3.clone()).await {
                Err(err) => {
                    error!("Error in write task: {err}");
                    cancel3.cancel();
                }
                Ok(_) => cancel3.cancel(),
            }
        });

        // This is the task for the transfer frame buffer. The buffer is expected to be sent
        // when it is either full or the latency limit has been reached. This task reads
        // from the buffer and sends it
        tokio::spawn(async move {
            let mut rand = SeedableRng::from_entropy();
            let continuity = AtomicI32::new(-1);

            loop {
                let frames = buf_receiver.recv().await;

                if !frames.is_empty() {
                    // prepare the frames
                    match convert_frames(&config4, &raf_config4, &mut rand, &continuity, frames) {
                        Err(err) => {
                            error!("Error encoding TM Frames: {err}");
                            continue;
                        }
                        Ok(trans) => {
                            let pdu = SleMsg::PDU(SlePdu::SleRafTransferBuffer(trans));
                            if let Err(err) = sender3.send(pdu).await {
                                error!("Error sending TM Frames: {err}");
                                cancel4.cancel();
                            }
                        }
                    }
                }
            }
        });

        self.read_handle = Some(read_handle);
        self.write_handle = Some(write_handle);

        Ok(())
    }

    pub async fn wait_for_termination(&mut self) {
        // we only want to return, when the tasks have finished. So we await the handles.
        // Unfortunately, we have no scoped tasks for async.
        if let Some(hdl) = self.read_handle.take() {
            let _ = hdl.await;
        }
        if let Some(hdl) = self.write_handle.take() {
            let _ = hdl.await;
        }
    }

    /// Send a TM Transfer Frame via an established SLE service
    pub async fn send_frame(&self, frame: SleFrame) -> Result<(), String> {
        if self.raf_state.load(Ordering::Relaxed) != RAFState::Active {
            return Err(format!(
                "Tried to send Frame while not in active state: {}",
                self.raf_config.sii
            ));
        }

        match &self.buffer_sender {
            Some(chan) => {
                chan.send(DataBufferElement::Frame(frame)).await;
                Ok(())
            }
            None => Err(format!(
                "Tried to send TM Frame when no channel was established on {}",
                self.raf_config.sii
            )),
        }
    }

    pub async fn notify_sync_loss(
        &self,
        time: &Time,
        carrier_lock_status: LockStatus,
        subcarrier_lock_status: LockStatus,
        symbol_sync_lock_status: LockStatus,
    ) -> Result<(), String> {
        if self.raf_state.load(Ordering::Relaxed) != RAFState::Active {
            return Err(format!(
                "Tried to send Notification while not in active state: {}",
                self.raf_config.sii
            ));
        }

        match &self.buffer_sender {
            Some(chan) => {
                let time = crate::types::sle::Time::CcsdsFormat(to_ccsds_time(time)?);
                chan.send(DataBufferElement::Notification(
                    Notification::LossFrameSync {
                        time: time,
                        carrier_lock_status: (carrier_lock_status as i32).into(),
                        subcarrier_lock_status: (subcarrier_lock_status as i32).into(),
                        symbol_sync_lock_status: (symbol_sync_lock_status as i32).into(),
                    },
                ))
                .await;

                Ok(())
            }
            None => Err(format!(
                "Tried to send TM Frame when no channel was established on {}",
                self.raf_config.sii
            )),
        }
    }

    pub async fn wait_active(&mut self) -> bool {
        self.state_watch
            .wait_for(|val| *val == RAFState::Active)
            .await
            .is_ok()
    }

    pub async fn stop(&self) {
        self.cancel_token.cancel();
    }
}

async fn read_context_message(
    args: &mut Args,
    server_startup_interval: Duration,
) -> Result<(u16, u16), String> {
    select! {
        res = TMLMessage::async_read(&mut args.rx) => {
            match res {
                Err(err) => { return Err(format!("Error reading TML Context Message: {err}")); }
                Ok(msg) => {
                    debug!("Read TML message {msg:?}");

                    let (interval, dead_factor) = msg.check_context()?;
                    let tml = &args.common_config.tml;

                    if interval < tml.min_heartbeat || interval > tml.max_heartbeat {
                        return Err(format!("Error: TML Context message interval ({interval}) is out of allowed range ([{}, {}])", tml.min_heartbeat, tml.max_heartbeat));
                    }

                    if dead_factor < tml.min_dead_factor || dead_factor > tml.max_dead_factor {
                        return Err(format!("Error: TML Context message dead factor ({dead_factor}) is out of allowed range ([{}, {}])", tml.min_dead_factor, tml.max_dead_factor));
                    }

                    return Ok((interval, dead_factor));
                }
            }
        }
        _ = tokio::time::sleep(server_startup_interval) => {
            return Err("Timeout waiting for TML Context Message".to_string());
        }
    };
}

async fn read_pdus(args: &mut Args) -> Result<(), String> {
    let timeout = Duration::from_secs(args.interval as u64 * args.dead_factor as u64);

    loop {
        select! {
            biased;

            res = TMLMessage::async_read(&mut args.rx) => {
                match res {
                    Err(err) => {
                        return Err(format!("Error reading SLE TML Message: {err}"));
                    }
                    Ok(msg) => {
                        if msg.is_heartbeat() {
                            debug!("SLE TML heartbeat received");
                        }
                        else
                        {
                            parse_sle_message(args, &msg).await;
                        }
                    }
                }
            }
            _ = tokio::time::sleep(timeout) => {
                return Err(format!("Timeout waiting for heartbeat message"));
            }
            _ = args.cancel_token.cancelled() => {
                debug!("RAF provider for {} read loop has been cancelled", args.raf_config.sii);
                return Ok(());
            }
        };
    }
}

async fn parse_sle_message(args: &mut Args, msg: &TMLMessage) {
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    match res {
        Ok(SlePdu::SlePeerAbort { diagnostic: diag }) => {
            warn!("Received Peer Abort with diagnostic: {:?}", diag);
            args.cancel_token.cancel();
        }
        Ok(pdu) => {
            // then continue processing
            process_sle_pdu(args, &pdu).await;
        }
        Err(err) => {
            error!("Error on decoding SLE PDU: {err}");
        }
    }
}

fn check_authentication(config: &CommonConfig, state: InternalState, pdu: &SlePdu) -> bool {
    match config.auth_type {
        SleAuthType::AuthNone =>
        // in case we have not authentication configured, all is good
        {
            true
        }
        // for AUTH_BIND, we need to only check the BIND and BIND RETURN invocations
        SleAuthType::AuthBind => check_bind(config, pdu),
        SleAuthType::AuthAll => check_all(config, state, pdu),
    }
}

fn check_bind(config: &CommonConfig, pdu: &SlePdu) -> bool {
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
            Credentials::Used(creds) => check_peer(config, creds, initiator_identifier, "BIND"),
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

fn check_all(config: &CommonConfig, state: InternalState, pdu: &SlePdu) -> bool {
    let credentials = pdu.get_credentials();

    match pdu {
        SlePdu::SleRafTransferBuffer(buffer) => {
            for frame in buffer {
                let res = check_buffer_credential(config, state.clone(), frame);
                if !res {
                    return false;
                }
            }
            return true;
        }
        SlePdu::SleBindInvocation {
            invoker_credentials,
            initiator_identifier,
            ..
        } => match invoker_credentials {
            Credentials::Unused => {
                error!("BIND Authentication failed: AUTH_BIND requested, but BIND invocation does not contain credentials");
                return false;
            }
            Credentials::Used(creds) => check_peer(config, creds, initiator_identifier, "BIND"),
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
        _ => match credentials {
            Some(creds) => match creds {
                Credentials::Unused => {
                    error!("Authentication failed, no credentials provided");
                    return false;
                }
                Credentials::Used(isp1) => {
                    let name;
                    {
                        let lock = state.lock().expect("Mutex lock failed");
                        name = lock.user().clone();
                    }
                    let op_name = pdu.operation_name();
                    return check_peer(config, isp1, &name, op_name);
                }
            },
            None => {
                return true;
            }
        },
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
                name = lock.user().clone();
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

async fn process_sle_pdu(args: &mut Args, pdu: &SlePdu) {
    match pdu {
        SlePdu::SleBindInvocation {
            invoker_credentials: _,
            initiator_identifier,
            responder_port_identifier,
            service_type,
            version_number,
            service_instance_identifier,
        } => {
            // check authentication
            if !check_authentication(&args.common_config, args.state.clone(), &pdu) {
                error!("SLE PDU failed authentication");

                let credentials = new_credentials(&args.common_config, &mut args.rand);

                // send back a negative acknowledge
                let ret = SlePdu::SleBindReturn {
                    performer_credentials: credentials,
                    responder_identifier: VisibleString::new(args.raf_config.provider.clone()),
                    result: BindResult::BindDiag(BindDiagnostic::AccessDenied),
                };

                let _ = args.chan.send(SleMsg::PDU(ret)).await;
                return;
            }

            // Now do the processing of the BIND
            if let Err(err) = process_bind(
                args,
                initiator_identifier,
                responder_port_identifier,
                service_type,
                version_number,
                service_instance_identifier,
            )
            .await
            {
                error!("Error processing BIND: {err}");
            }
        }
        SlePdu::SleUnbindInvocation {
            invoker_credentials: _,
            unbind_reason,
        } => {
            // check authentication
            if !check_authentication(&args.common_config, args.state.clone(), &pdu) {
                warn!("SLE PDU UNBIND failed authentication, ignoring PDU...");

                return;
            }

            // Processing of the UNBIND
            if let Err(err) = process_unbind(args, unbind_reason).await {
                error!("Error processing UNBIND: {err}");
            }
        }
        SlePdu::SlePeerAbort { diagnostic } => {
            let _ = process_peer_abort(args, diagnostic).await;
        }
        SlePdu::SleRafStartInvocation {
            invoker_credentials: _,
            invoke_id,
            start_time,
            stop_time,
            requested_frame_quality,
        } => {
            // check authentication
            if !check_authentication(&args.common_config, args.state.clone(), &pdu) {
                error!("RAF START PDU failed authentication");

                let credentials = new_credentials(&args.common_config, &mut args.rand);

                // send back a negative acknowledge
                let ret = SlePdu::SleRafStartReturn {
                    performer_credentials: credentials,
                    invoke_id: *invoke_id,
                    result: RafStartReturnResult::NegativeResult(DiagnosticRafStart::Common(
                        Diagnostics::OtherReason,
                    )),
                };

                let _ = args.chan.send(SleMsg::PDU(ret)).await;
                return;
            }

            // Now do the processing of the BIND
            if let Err(err) = process_start(
                args,
                invoke_id,
                start_time,
                stop_time,
                requested_frame_quality,
            )
            .await
            {
                error!("Error processing BIND: {err}");
            }
        }
        SlePdu::SleRafStopInvocation {
            invoker_credentials: _,
            invoke_id,
        } => {
            // check authentication
            if !check_authentication(&args.common_config, args.state.clone(), &pdu) {
                error!("RAF STOP PDU failed authentication");

                let credentials = new_credentials(&args.common_config, &mut args.rand);

                // send back a negative acknowledge
                let ret = SlePdu::SleAcknowledgement {
                    credentials: credentials,
                    invoke_id: *invoke_id,
                    result: SleResult::NegativeResult(Diagnostics::OtherReason),
                };

                let _ = args.chan.send(SleMsg::PDU(ret)).await;
                return;
            }

            // Now do the processing of the BIND
            if let Err(err) = process_stop(args, *invoke_id).await {
                error!("Error processing STOP: {err}");
            }
        }
        SlePdu::SleRafGetParameterIncovation {
            invoker_credentials: _,
            invoke_id,
            raf_parameter,
        } => {
            // check authentication
            if !check_authentication(&args.common_config, args.state.clone(), &pdu) {
                error!("RAF GET PARAMETER PDU failed authentication");

                let credentials = new_credentials(&args.common_config, &mut args.rand);

                // send back a negative acknowledge
                let ret = SlePdu::SleAcknowledgement {
                    credentials: credentials,
                    invoke_id: *invoke_id,
                    result: SleResult::NegativeResult(Diagnostics::OtherReason),
                };

                let _ = args.chan.send(SleMsg::PDU(ret)).await;
                return;
            }

            // Now do the processing of the BIND
            if let Err(err) = process_get_parameter(args, *invoke_id, *raf_parameter).await {
                error!("Error processing GET PARAMETER: {err}");
            }
        }
        pdu => {
            info!("Not yet implemented: processing for PDU: {:?}", pdu);
        }
    }
}

async fn write_task(
    common_config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    mut tx: OwnedWriteHalf,
    receiver: &mut Receiver<SleMsg>,
    cancel: CancellationToken,
) -> Result<(), String> {
    let mut timeout = common_config.tml.heartbeat;

    loop {
        select! {
                res = receiver.recv() => {
                    match res {
                        Some(SleMsg::Stop) => {
                            debug!("Stop requested");
                            return Ok(());
                        }
                        Some(SleMsg::BindReturn(pdu, hb, _dead_factor)) => {
                            timeout = hb;
                            send_sle_msg(pdu, &mut tx).await?;
                        }
                        Some(SleMsg::PDU(pdu)) => {
                            send_sle_msg(pdu, &mut tx).await?;
                        }
                        None => {
                            debug!("Send channel has been closed, returning...");
                            return Ok(());
                        }
                    }
                },
                _ = tokio::time::sleep(Duration::from_secs(timeout as u64)) => {
                    // we have a timeout, so send a heartbeat message
                    if let Err(err) = TMLMessage::heartbeat_message().write_to_async(&mut tx).await {
                        return Err(format!("Error sending SLE TML hearbeat message: {}", err));
                    }
                },
                _ = cancel.cancelled() => {
                    debug!("RAF provider for {} has been cancelled (write task)", raf_config.sii);
                    // if let Ok(tml) = process_sle_msg(SlePdu::SlePeerAbort{ diagnostic: PeerAbortDiagnostic::OtherReason}, raf_state3.clone()) {
                    //     let _ = tml.write_to_async(&mut tx).await;
                    // }
                    return Ok(());
                }
        }
    }
}

async fn send_sle_msg(pdu: SlePdu, tx: &mut OwnedWriteHalf) -> Result<(), String> {
    match rasn::der::encode(&pdu) {
        Err(err) => {
            return Err(format!("Error encoding PDU to ASN1: {err}"));
        }
        Ok(val) => {
            let tml = TMLMessage::new_with_data(val);
            if let Err(err) = tml.write_to_async(tx).await {
                return Err(format!("Could not send SLE PDU: {err}"));
            }
        }
    }
    Ok(())
}

async fn process_bind(
    args: &mut Args,
    initiator_identifier: &AuthorityIdentifier,
    responder_port_identifier: &PortId,
    service_type: &Integer,
    version_number: &VersionNumber,
    service_instance_identifier: &ServiceInstanceIdentifier,
) -> Result<(), String> {
    // First perform all checks to see if the BIND request is legal

    let app_id = ApplicationIdentifier::try_from(service_type);
    let app_chk = match app_id {
        Ok(ApplicationIdentifier::RtnAllFrames) => true,
        Ok(_) => false,
        Err(_) => false,
    };

    let (bind_result, version) = if !app_chk {
        (
            BindResult::BindDiag(BindDiagnostic::ServiceTypeNotSupported),
            SleVersion::V5,
        )
    } else {
        match SleVersion::try_from(*version_number as u8) {
            Err(_) => (
                BindResult::BindDiag(BindDiagnostic::VersionNotSupported),
                SleVersion::V5,
            ),
            Ok(version) => {
                match service_instance_identifier_to_string(service_instance_identifier) {
                    Err(_) => (
                        BindResult::BindDiag(BindDiagnostic::NoSuchServiceInstance),
                        SleVersion::V5,
                    ),
                    Ok(sii) => {
                        if sii != args.raf_config.sii {
                            (
                                BindResult::BindDiag(BindDiagnostic::NoSuchServiceInstance),
                                SleVersion::V5,
                            )
                        } else {
                            if responder_port_identifier.value != args.raf_config.responder_port {
                                (
                                    BindResult::BindDiag(
                                        BindDiagnostic::SiNotAccessibleToThisInitiator,
                                    ),
                                    SleVersion::V5,
                                )
                            } else {
                                // Ok, BIND is ok, so process the request now
                                let mut lock = args.state.lock().unwrap();
                                match lock.process_bind(initiator_identifier, version) {
                                    Ok(()) => {
                                        let _ = args.state_watch.send(lock.state());
                                        (BindResult::BindOK(version as u16), version)
                                    }
                                    Err(err) => {
                                        error!("{err}");
                                        (
                                            BindResult::BindDiag(BindDiagnostic::AlreadyBound),
                                            SleVersion::V5,
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    match bind_result {
        BindResult::BindOK(_) => {
            info!("BIND on {} successful", args.raf_config.sii);
        }
        BindResult::BindDiag(diag) => {
            return Err(format!("BIND on {} failed: {diag:?}", args.raf_config.sii));
        }
    };

    // Create a bind return PDU
    let credentials = new_credentials(&args.common_config, &mut args.rand);
    let pdu = SlePdu::SleBindReturn {
        performer_credentials: credentials,
        responder_identifier: VisibleString::new(args.raf_config.provider.clone()),
        result: bind_result,
    };

    // Send it
    let _ = args
        .chan
        .send(SleMsg::BindReturn(pdu, args.interval, args.dead_factor))
        .await;

    // Now, notify the application that the bind was successful
    {
        //let lock = args.app_notifier.lock().unwrap();
        //lock.bind_succeeded(&initiator_identifier.value, &args.raf_config.sii, version);
        args.app_notifier.bind_succeeded(
            &initiator_identifier.value,
            &args.raf_config.sii,
            version,
        );
    }

    Ok(())
}

async fn process_unbind(args: &mut Args, reason: &Integer) -> Result<(), String> {
    let reason = UnbindReason::try_from(reason);

    let reason = match reason {
        Ok(reason) => reason,
        Err(err) => {
            warn!("Error converting UNBIND reason: {err}");
            UnbindReason::Other
        }
    };

    // Ok, BIND is ok, so process the request now
    {
        let mut lock = args.state.lock().unwrap();
        let _ = lock.process_unbind(reason);
        let _ = args.state_watch.send(lock.state());
    }

    // Create a unbind return PDU
    let credentials = new_credentials(&args.common_config, &mut args.rand);
    let pdu = SlePdu::SleUnbindReturn {
        responder_credentials: credentials,
        result: (),
    };

    // Send it
    let _ = args.chan.send(SleMsg::PDU(pdu)).await;

    // Now, notify the application that the bind was successful
    {
        //let lock = args.app_notifier.lock().unwrap();
        //lock.unbind_succeeded(&args.raf_config.sii, reason);
        args.app_notifier
            .unbind_succeeded(&args.raf_config.sii, reason);
    }

    Ok(())
}

async fn process_peer_abort(
    args: &mut Args,
    diagnostic: &PeerAbortDiagnostic,
) -> Result<(), String> {
    warn!("Received PEER ABORT with diagnostic {diagnostic:?}");

    {
        let mut lock = args.state.lock().unwrap();
        lock.peer_abort(diagnostic);
    }

    // Now, notify the application that the bind was successful
    {
        //let lock = args.app_notifier.lock().unwrap();
        //lock.peer_abort(&args.raf_config.sii, diagnostic);
        args.app_notifier
            .peer_abort(&args.raf_config.sii, diagnostic);
    }

    Ok(())
}

async fn process_start(
    args: &mut Args,
    invoke_id: &InvokeId,
    start_time: &ConditionalTime,
    stop_time: &ConditionalTime,
    requested_frame_quality: &Integer,
) -> Result<(), String> {
    // First perform all checks to see if the BIND request is legal

    let frame_qual = RequestedFrameQuality::try_from(requested_frame_quality);
    let (diag, frame_qual, start, stop) = match frame_qual {
        Ok(frame_qual) => {
            // check the start time, must be present in OFFLINE mode
            if args.raf_config.mode == RafDeliveryMode::RtnOffline && start_time.is_null() {
                (
                    RafStartReturnResult::NegativeResult(DiagnosticRafStart::Specific(
                        SpecificDiagnosticRafStart::MissingTimeValue,
                    )),
                    frame_qual,
                    None,
                    None,
                )
            }
            // also check that stop time is present in OFFLINE mode
            else if args.raf_config.mode == RafDeliveryMode::RtnOffline && stop_time.is_null() {
                (
                    RafStartReturnResult::NegativeResult(DiagnosticRafStart::Specific(
                        SpecificDiagnosticRafStart::MissingTimeValue,
                    )),
                    frame_qual,
                    None,
                    None,
                )
            } else {
                // check, if we can convert the start time correctly
                match from_conditional_ccsds_time(start_time) {
                    Err(err) => {
                        error!("Could not convert start time from RAF START: {err}");
                        (
                            RafStartReturnResult::NegativeResult(DiagnosticRafStart::Specific(
                                SpecificDiagnosticRafStart::InvalidStartTime,
                            )),
                            frame_qual,
                            None,
                            None,
                        )
                    }
                    Ok(start) => {
                        // check, if we can convert the stop time correctly
                        match from_conditional_ccsds_time(stop_time) {
                            Err(err) => {
                                error!("Could not convert stop time from RAF START: {err}");
                                (
                                    RafStartReturnResult::NegativeResult(
                                        DiagnosticRafStart::Specific(
                                            SpecificDiagnosticRafStart::InvalidStopTime,
                                        ),
                                    ),
                                    frame_qual,
                                    None,
                                    None,
                                )
                            }
                            Ok(stop) => {
                                // Check if the times are in order
                                match (start.clone(), stop.clone()) {
                                    (Some(start), Some(stop)) => {
                                        if start > stop {
                                            error!(
                                                "RAF START: start time is greater than stop time"
                                            );
                                            (RafStartReturnResult::NegativeResult(DiagnosticRafStart::Specific(SpecificDiagnosticRafStart::InvalidStartTime)), frame_qual, None, None)
                                        } else {
                                            // we are good
                                            (
                                                RafStartReturnResult::PositiveResult,
                                                frame_qual,
                                                Some(start),
                                                Some(stop),
                                            )
                                        }
                                    }
                                    _ => (
                                        RafStartReturnResult::PositiveResult,
                                        frame_qual,
                                        start,
                                        stop,
                                    ),
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(err) => {
            error!("Error parsing Requested Frame Quality from RAF START: {err}");
            (
                RafStartReturnResult::NegativeResult(DiagnosticRafStart::Specific(
                    SpecificDiagnosticRafStart::UnableToComply,
                )),
                RequestedFrameQuality::AllFrames,
                None,
                None,
            )
        }
    };

    let credentials = new_credentials(&args.common_config, &mut args.rand);
    match &diag {
        RafStartReturnResult::PositiveResult => {
            // Ok, START is ok, so process the request now
            let diag = {
                let mut lock = args.state.lock().unwrap();
                match lock.process_start(start, stop, frame_qual) {
                    Ok(()) => {
                        let _ = args.state_watch.send(lock.state());
                        RafStartReturnResult::PositiveResult
                    }
                    Err(err) => {
                        error!("{err}");
                        RafStartReturnResult::NegativeResult(DiagnosticRafStart::Specific(
                            SpecificDiagnosticRafStart::UnableToComply,
                        ))
                    }
                }
            };

            // send a return
            let pdu = SlePdu::SleRafStartReturn {
                performer_credentials: credentials,
                invoke_id: *invoke_id,
                result: diag,
            };

            // Send it
            let _ = args.chan.send(SleMsg::PDU(pdu)).await;

            if let RafStartReturnResult::PositiveResult = diag {
                // Now, notify the application that the bind was successful
                {
                    //let lock = args.app_notifier.lock().unwrap();
                    //lock.start_succeeded(&args.raf_config.sii);
                    args.app_notifier.start_succeeded(&args.raf_config.sii);
                }
            }
        }
        RafStartReturnResult::NegativeResult(_) => {
            // send a return
            let pdu = SlePdu::SleRafStartReturn {
                performer_credentials: credentials,
                invoke_id: *invoke_id,
                result: diag,
            };
            // Send it
            let _ = args.chan.send(SleMsg::PDU(pdu)).await;
        }
    }
    Ok(())
}

async fn process_stop(args: &mut Args, invoke_id: u16) -> Result<(), String> {
    // Ok, BIND is ok, so process the request now
    let diag = {
        {
            let mut lock = args.state.lock().unwrap();
            match lock.process_stop() {
                Err(err) => {
                    error!("{err}");
                    SleResult::NegativeResult(Diagnostics::OtherReason)
                }
                Ok(()) => {
                    let _ = args.state_watch.send(lock.state());
                    SleResult::PositiveResult
                }
            }
        }
    };

    // Create a SLE Ack PDU
    let credentials = new_credentials(&args.common_config, &mut args.rand);
    let pdu = SlePdu::SleAcknowledgement {
        credentials: credentials,
        invoke_id: invoke_id,
        result: diag,
    };

    // Send it
    let _ = args.chan.send(SleMsg::PDU(pdu)).await;

    if let SleResult::PositiveResult = diag {
        //let lock = args.app_notifier.lock().unwrap();
        //lock.stop_succeeded(&args.raf_config.sii);
        args.app_notifier.stop_succeeded(&args.raf_config.sii);
    }

    Ok(())
}

fn new_credentials(config: &CommonConfig, rand: &mut StdRng) -> Credentials {
    match config.auth_type {
        SleAuthType::AuthNone => Credentials::Unused,
        SleAuthType::AuthAll | SleAuthType::AuthBind => {
            let isp1 = ISP1Credentials::new(
                config.hash_to_use,
                &rs_space_core::time::Time::now(TimeEncoding::CDS8),
                rand.gen(),
                &config.authority_identifier,
                &config.password,
            );
            Credentials::Used(isp1)
        }
    }
}

fn convert_frames(
    config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    rand: &mut StdRng,
    continuity: &AtomicI32,
    frames: Vec<DataBufferElement>,
) -> Result<RafTransferBuffer, String> {
    // allocate a vector for the output
    let mut res = RafTransferBuffer::with_capacity(frames.len());

    // we loop over the frames
    for elem in frames {
        match elem {
            DataBufferElement::Frame(frame) => {
                // create a new credentials value for the frame
                let credentials = new_credentials(config, rand);
                // convert the ERT into the SLE form
                let time = to_ccsds_time(&frame.earth_receive_time)?;

                // Add the converted frame to the vector
                res.push(FrameOrNotification::AnnotatedFrame(
                    RafTransferDataInvocation {
                        invoker_credentials: credentials,
                        earth_receive_time: crate::types::sle::Time::CcsdsFormat(time),
                        antenna_id: raf_config.antenna_id.clone(),
                        data_link_continuity: continuity.load(Ordering::Relaxed),
                        delivered_frame_quality: frame.delivered_frame_quality as i32,
                        private_annotation: PrivateAnnotation::Null,
                        data: frame.data,
                    },
                ));
            }
            DataBufferElement::Notification(notif) => {
                // create a new credentials value for the frame
                let credentials = new_credentials(config, rand);

                res.push(FrameOrNotification::SyncNotification(
                    RafSyncNotifyInvocation {
                        invoker_credentials: credentials,
                        notification: notif,
                    },
                ));
            }
        }
    }

    Ok(res)
}

async fn process_get_parameter(
    args: &mut Args,
    invoke_id: u16,
    parameter_name: i64,
) -> Result<(), String> {
    // Ok, BIND is ok, so process the request now
    let diag = match crate::types::sle::ParameterName::try_from(parameter_name) {
        Err(_err) => RafGetReturnResult::NegativeResult(DiagnosticRafGet::Specific(
            SpecificDiagnosticRafGet::UnknownParameter,
        )),
        Ok(param) => {
            let lock = args.state.lock().unwrap();
            lock.process_get_param(param)
        }
    };

    // Create a SLE Ack PDU
    let credentials = new_credentials(&args.common_config, &mut args.rand);
    
    let pdu = SlePdu::SleRafGetParameterReturn {
        performer_credentials: credentials,
        invoke_id: invoke_id,
        result: diag,
    };
    debug!("Get Parameter received: returning: {:?}", pdu);

    // Send it
    let _ = args.chan.send(SleMsg::PDU(pdu)).await;

    Ok(())
}
