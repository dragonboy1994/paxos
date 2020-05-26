use crossbeam::channel::{unbounded, Receiver, Sender};

// all sender handles of the broadcast channel
#[derive(Clone)]
pub struct BroadcastSender<T>(Vec<Sender<T>>);

// all receiver handles of the broadcast channel
#[derive(Clone)]
pub struct BroadcastReceivers<T>(Vec<Receiver<T>>);

impl<T: std::clone::Clone> BroadcastSender<T> {
    // send data to all receivers
    // called by sender
    pub fn send(&self, data: T) {
        for receiver_id in 0..self.0.len() {
            self.0[receiver_id]
            .send(data.clone()).expect("Acceptor at the receive has been deactivated !!! No need to panic !!!");
        }
    }
}

impl<T: std::clone::Clone> BroadcastReceivers<T> {
    // return the receiver handles
    pub fn handle_split(self) -> Vec<Receiver<T>> {
        self.0
    }
}

// constructing the broadcast channel
pub fn construct<T: std::clone::Clone>(
    num_receivers: u32,
) -> (BroadcastSender<T>, BroadcastReceivers<T>) {
    let mut senders: Vec<Sender<T>> = Vec::new();
    let mut receivers: Vec<Receiver<T>> = Vec::new();

    for _ in 0..num_receivers {
        let (sender, receiver) = unbounded();
        senders.push(sender);
        receivers.push(receiver);
    }

    (BroadcastSender(senders), BroadcastReceivers(receivers))
}
