use tokio::{sync::Notify, time::timeout};

use std::{
    sync::{Arc, Mutex},
    time::Duration, marker::PhantomData,
};

pub struct TimedBuffer<T> {
    t: PhantomData<T>
}

pub struct Sender<T> {
    capacity: usize,
    values: Arc<Mutex<Vec<T>>>,
    notify_read: Arc<Notify>,
    notify_write: Arc<Notify>,
}

pub struct Receiver<T> {
    capacity: usize,
    timeout: Duration,
    values: Arc<Mutex<Vec<T>>>,
    notify_read: Arc<Notify>,
    notify_write: Arc<Notify>,
}

impl<T> Sender<T> {
    pub async fn send(&self, msg: T) {
        let mut lock = self.values.lock().unwrap();
        if lock.len() < self.capacity {
            lock.push(msg);
            return;
        } else {
            drop(lock);
            self.notify_read.notify_one();
            self.notify_write.notified().await
        }
    }
}

impl<T> Receiver<T> {
    pub async fn recv(&self) -> Vec<T> {
        loop {
            match timeout(self.timeout, self.notify_read.notified()).await {
                Err(_) => {
                    let mut lock = self.values.lock().unwrap();
                    if lock.is_empty() {
                        continue;
                    } else {
                        let mut res = Vec::with_capacity(self.capacity);
                        std::mem::swap(&mut *lock, &mut res);
                        drop(lock);

                        self.notify_write.notify_one();
                        if res.is_empty() {
                            continue;
                        }
                        return res;
                    }
                }
                Ok(()) => {
                    // we got a signal, return the value
                    let mut lock = self.values.lock().unwrap();
                    let mut res = Vec::with_capacity(self.capacity);
                    std::mem::swap(&mut *lock, &mut res);

                    self.notify_write.notify_one();
                    return res;
                }
            }
        }
    }
}

impl<T> TimedBuffer<T> {
    pub fn new(capacity: usize, timeout: Duration) -> (Sender<T>, Receiver<T>) {

        let vals = Arc::new(Mutex::new(Vec::with_capacity(capacity)));
        let vals2 = vals.clone();
        
        let read_notif = Arc::new(Notify::new());
        let read_notif2 = read_notif.clone();

        let write_notif = Arc::new(Notify::new());
        let write_notif2 = write_notif.clone();

        let sender = Sender {
            capacity,
            values: vals,
            notify_read: read_notif,
            notify_write: write_notif
        };

        let receiver = Receiver {
            capacity,
            timeout,
            values: vals2,
            notify_read: read_notif2, 
            notify_write: write_notif2
        };

        (sender, receiver)
    }

    // pub async fn send(&self, msg: T) {
    //     let mut lock = self.values.lock().unwrap();
    //     if lock.len() < self.capacity {
    //         lock.push(msg);
    //         return;
    //     } else {
    //         drop(lock);
    //         self.notify_read.notify_one();
    //         self.notify_write.notified().await
    //     }
    // }

    // pub async fn recv(&self) -> Vec<T> {
    //     loop {
    //         match timeout(self.timeout, self.notify_read.notified()).await {
    //             Err(_) => {
    //                 let mut lock = self.values.lock().unwrap();
    //                 if lock.is_empty() {
    //                     continue;
    //                 } else {
    //                     let mut res = Vec::with_capacity(self.capacity);
    //                     std::mem::swap(&mut *lock, &mut res);
    //                     drop(lock);

    //                     self.notify_write.notify_one();
    //                     if res.is_empty() {
    //                         continue;
    //                     }
    //                     return res;
    //                 }
    //             }
    //             Ok(()) => {
    //                 // we got a signal, return the value
    //                 let mut lock = self.values.lock().unwrap();
    //                 let mut res = Vec::with_capacity(self.capacity);
    //                 std::mem::swap(&mut *lock, &mut res);

    //                 self.notify_write.notify_one();
    //                 return res;
    //             }
    //         }
    //     }
    // }
}
