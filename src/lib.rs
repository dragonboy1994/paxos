mod replica;
mod leader;
//mod client;
mod broadcast_channel;

//use crate::replica;
//use crate::leader;
use crate::broadcast_channel::{BroadcastSender, BroadcastReceivers};


use crossbeam::channel::{Sender, Receiver};
use log::info;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;



// contains the handles for control channels for replica and leaders
pub struct SystemHandles {
    replica_control_chan_sender: broadcast_channel::BroadcastSender<replica::ControlSignal>,
    leader_control_chan_sender: broadcast_channel::BroadcastSender<leader::ControlSignal>,
}



impl SystemHandles {
    pub fn new(replica_count: usize, leader_count: usize) -> SystemHandles {



        // get the broadcast control channels for the replicas
        let (replica_control_chan_sender, replica_control_chan_receivers) 
                    = broadcast_channel::construct::<replica::ControlSignal>(replica_count.clone() as u8);

        
        // get the broadcast control channels for the leaders
        let (leader_control_chan_sender, leader_control_chan_receivers) 
                    = broadcast_channel::construct::<leader::ControlSignal>(leader_count.clone() as u8);


        // get the vector of handles and then reverse it
        // reversing eases the assignment of the handles to the replicas and leaders by pop later
        let mut split_replica_control_chan_receivers = replica_control_chan_receivers.handle_split();
        let mut split_leader_control_chan_receivers = leader_control_chan_receivers.handle_split();
        split_replica_control_chan_receivers.reverse();
        split_leader_control_chan_receivers.reverse();


        // collecting all broadcast channel receiver handles for leaders while iterating over replicas
        let mut hashmap_replica_leader_broadcast_chan_receivers: HashMap<usize, Vec<Receiver<u8>>> = HashMap::new();





        // iterate over each replica 
        for replica_id in 0..replica_count {

            // get the broadcast channel from curent replica to all leaders
            let (replica_leader_broadcast_chan_sender, replica_leader_broadcast_chan_receivers) 
                    = broadcast_channel::construct::<u8>(leader_count.clone() as u8);

        
            // build the replica
            let replica_context = replica::new( replica_id as u8,
                                                replica_leader_broadcast_chan_sender,
                                                split_replica_control_chan_receivers.pop().unwrap()
                                            );
            
            // start the replica in paused mode
            replica_context.start();

            let mut split_replica_leader_broadcast_chan_receivers 
                                        = replica_leader_broadcast_chan_receivers.handle_split();
            split_replica_leader_broadcast_chan_receivers.reverse();

            // collect the receiver handles for leaders in the hashmap
            hashmap_replica_leader_broadcast_chan_receivers.insert(
                replica_id,
                split_replica_leader_broadcast_chan_receivers
            );

        }
    





        // iterate over each leader
        for leader_id in 0..leader_count {

            // collect the receiver handles of the broadcast channels from all the replicas
            let mut replica_leader_broadcast_chan_receivers: Vec<Receiver<u8>> = Vec::new(); 

            for replica_id in 0..replica_count {
                // retrieving the entry corresponding to replica_id
                // this approach taken because HashMap doesn't implement IndexMut trait
                let mut hashmap_entry_replica_id = hashmap_replica_leader_broadcast_chan_receivers
                                                .remove(&replica_id)
                                                .unwrap();

                // update
                replica_leader_broadcast_chan_receivers
                    .push(hashmap_entry_replica_id.pop().unwrap());

                // insert back the entry corresponding to replica_id
                hashmap_replica_leader_broadcast_chan_receivers.insert(
                    replica_id,
                    hashmap_entry_replica_id
                );
            }

            // build the leader
            let leader_context = leader::new(leader_id as u8, 
                                            replica_leader_broadcast_chan_receivers, 
                                            split_leader_control_chan_receivers.pop().unwrap());
            
            // start the leader in paused mode
            leader_context.start(); 
        }


        

        SystemHandles{
            replica_control_chan_sender,
            leader_control_chan_sender,
        }
    }



    pub fn activate_broadcast(&self, num_broadcasts: u8, replica_count: u8) {

        // activating both replica and leaders; broadcast will start now
        self.replica_control_chan_sender.send(replica::ControlSignal::Run(num_broadcasts));
        self.leader_control_chan_sender.send(leader::ControlSignal::Run(replica_count*num_broadcasts));

    
        thread::sleep(Duration::from_secs(1));
        println!("Exit signal sent; hopefully, leader and replica in paused mode");
        self.replica_control_chan_sender.send(replica::ControlSignal::Exit);
        self.leader_control_chan_sender.send(leader::ControlSignal::Exit);
        thread::sleep(Duration::from_secs(1));

    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let replica_count = 3 as usize;
        let leader_count = 5 as usize; 
        let num_msgs = 5u8;
        let system_handles = SystemHandles::new(replica_count, leader_count);
        system_handles.activate_broadcast(num_msgs, replica_count as u8);
        assert_eq!(2 + 2, 4);
    }
}
