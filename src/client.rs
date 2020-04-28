use std::thread;
use crossbeam::channel::{unbounded, Sender, Receiver, TryRecvError};
use std::time::Duration;

use crate::broadcast_channel::BroadcastSender;



enum OperatingState {
    Paused,
    Run(u8),
    Exit,
} 


#[derive(Clone)]
pub enum ControlSignal {
    Paused,
    Run(u8),
    Exit,
}




pub struct Context {

    // 
    client_replica_


}

