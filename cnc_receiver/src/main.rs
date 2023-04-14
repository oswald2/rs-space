#[allow(unused)]

use rs_space_core::ccsds_packet::FastCcsdsPacket;

pub mod packet_processor;

use tokio::io::BufReader;
use tokio::io::{Error, ErrorKind};
use tokio::net::{TcpListener, TcpStream};

use log::{error, info, LevelFilter};
use log4rs::append::{console::ConsoleAppender, console::Target, file::FileAppender};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // initialise the logging
    let format_str = "{l} - {m}\n";
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(format_str)))
        .build("log/cnc_receiver.log")?;
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

    // Now start the whole processing
    info!("C&C Receiver started.");

    let listener = TcpListener::bind("127.0.0.1:10000").await.unwrap();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        info!("Client connected: {:?}", addr);

        tokio::spawn(async move {
            match process(socket).await {
                Err(err) => {
                    error!("C&C Processor returned error, closing connection: {}", err);
                }
                Ok(_) => {
                    info!("Terminating...");
                }
            }
        });
    }
}

async fn process(socket: TcpStream) -> Result<(), Error> {
    let mut reader = BufReader::new(socket);

    loop {
        let mut pkt = FastCcsdsPacket::new();
        match pkt.read_from_async(&mut reader).await {
            Ok(_) => match packet_processor::process_fast_packet(pkt) {
                Ok(_) => (),
                Err(err) => return Err(err),
            },
            Err(err) => {
                error!("Got error from reading socket: {:?}", err);
                return Err(err);
            }
        }
    }
}
