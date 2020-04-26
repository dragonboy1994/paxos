mod replica;
mod leader;

//use crate::replica;
//use crate::leader;

use crossbeam::channel::{unbounded, Receiver, Sender};
use log::info;
use std::thread;
use std::time::Duration;


pub struct SystemHandles {
    replica_control_chan_sender: Sender<replica::ControlSignal>,
    leader_control_chan_sender: Sender<leader::ControlSignal>,
}



impl SystemHandles {
    pub fn new(leader_count: usize) -> SystemHandles {
        let (broadcast_chan_sender, broadcast_chan_receiver) = unbounded();
        let (leader_control_chan_sender, leader_control_chan_receiver) = unbounded();
        
        let (replica_context, replica_control_chan_sender) = replica::new(broadcast_chan_sender);
        let leader_context = leader::new(0u8, broadcast_chan_receiver, leader_control_chan_receiver);

        // both in paused mode
        replica_context.start();
        leader_context.start();

        SystemHandles{
            replica_control_chan_sender,
            leader_control_chan_sender,
        }
    }



    pub fn activate_broadcast(&self, num_rounds: u8) {

        // activating both replica and leader; broadcast will start now
        self.replica_control_chan_sender.send(replica::ControlSignal::Run(num_rounds)).unwrap();
        self.leader_control_chan_sender.send(leader::ControlSignal::Run(num_rounds)).unwrap();

    
        thread::sleep(Duration::from_secs(10));
        println!("Exit signal sent; hopefully, leader and replica in paused mode");
        self.replica_control_chan_sender.send(replica::ControlSignal::Exit).unwrap();
        self.leader_control_chan_sender.send(leader::ControlSignal::Exit).unwrap();
        thread::sleep(Duration::from_secs(10));

    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let system_handles = SystemHandles::new(2 as usize);
        system_handles.activate_broadcast(20u8);
        assert_eq!(2 + 2, 4);
    }
}
