mod replica;
mod leader;
mod broadcast_channel;

//use crate::replica;
//use crate::leader;
use crate::broadcast_channel::{BroadcastSender, BroadcastReceivers};


use crossbeam::channel::Sender;
use log::info;
use std::thread;
use std::time::Duration;


// contains the handles for control channels for replica and leaders
pub struct SystemHandles {
    replica_control_chan_sender: Sender<replica::ControlSignal>,
    leader_control_chan_sender: broadcast_channel::BroadcastSender<leader::ControlSignal>,
}



impl SystemHandles {
    pub fn new(leader_count: usize) -> SystemHandles {

        // get the broadcast channels for replica and leaders
        let (broadcast_chan_sender, broadcast_chan_receivers) = broadcast_channel::construct::<u8>(leader_count.clone() as u8);

        // get the broadcast control channels for the leaders
        let (leader_control_chan_sender, leaders_control_chan_receivers) = broadcast_channel::construct::<leader::ControlSignal>(leader_count.clone() as u8);


        // build the replica 
        let (replica_context, replica_control_chan_sender) = replica::new(broadcast_chan_sender);
        // start the replica in paused state
        replica_context.start();



        // prepare the receiver handles in broadcast channels for distributing among the leaders
        let mut split_broadcast_chan_receivers = broadcast_chan_receivers.handle_split();
        let mut split_leaders_control_chan_receivers = leaders_control_chan_receivers.handle_split();

        // build the leaders
        // start the leaders in paused state
        for leader_id in (0..leader_count).rev() {
            let leader_context = leader::new(leader_id as u8, 
                                            split_broadcast_chan_receivers.pop().unwrap(), 
                                            split_leaders_control_chan_receivers.pop().unwrap());
            leader_context.start(); 
        }


        

        SystemHandles{
            replica_control_chan_sender,
            leader_control_chan_sender,
        }
    }



    pub fn activate_broadcast(&self, num_broadcasts: u8) {

        // activating both replica and leaders; broadcast will start now
        self.replica_control_chan_sender.send(replica::ControlSignal::Run(num_broadcasts)).unwrap();
        self.leader_control_chan_sender.send(leader::ControlSignal::Run(num_broadcasts));

    
        thread::sleep(Duration::from_secs(1));
        println!("Exit signal sent; hopefully, leader and replica in paused mode");
        self.replica_control_chan_sender.send(replica::ControlSignal::Exit).unwrap();
        self.leader_control_chan_sender.send(leader::ControlSignal::Exit);
        thread::sleep(Duration::from_secs(1));

    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let system_handles = SystemHandles::new(2 as usize);
        system_handles.activate_broadcast(5u8);
        assert_eq!(2 + 2, 4);
    }
}
