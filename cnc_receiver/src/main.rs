use rs_space_core::ccsds_packet::FastCcsdsPacket;
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:10000").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    let mut reader = BufReader::new(socket);

    let mut pkt = FastCcsdsPacket::new();
    match pkt.read_from_async(&mut reader).await {
        Ok(_) => todo!(),
        Err(err) => println!("Got error from readin socket: {:?}", err)
    }
}
