

use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::time::{sleep, Duration, timeout};

async fn process_buffer(mut receiver: Receiver<i32>, max_capacity: usize, timeout_duration: Duration) {
    let mut buffer = Vec::with_capacity(max_capacity);

    loop {
        match timeout(timeout_duration, receiver.recv()).await {
            Ok(Some(value)) => {
                buffer.push(value);
                if buffer.len() == max_capacity {
                    println!("Buffer full, delivering contents: {:?}", buffer);
                    buffer.clear(); // or send to a consumer
                }
            }
            Ok(None) => {
                println!("Channel closed");
                break;
            }
            Err(_) => {
                if !buffer.is_empty() {
                    println!("Timeout reached, delivering contents: {:?}", buffer);
                    buffer.clear(); // or send to a consumer
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (sender, receiver): (Sender<i32>, Receiver<i32>) = channel(100); // Channel with capacity 100
    let max_buffer_capacity = 10; // Max buffered items before delivery
    let buffer_timeout = Duration::from_secs(5); // Timeout duration

    tokio::spawn(async move {
        process_buffer(receiver, max_buffer_capacity, buffer_timeout).await;
    });

    // Simulated producer sending values into buffer
    for i in 0..200 {
        if sender.send(i).await.is_err() {
            println!("Buffer full or receiver dropped");
            break;
        }
        sleep(Duration::from_millis(100)).await; // Just for demonstration
    }
}
