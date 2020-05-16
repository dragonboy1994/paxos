#![allow(unused)]

mod acceptor;
mod broadcast_channel;
mod client;
mod leader;
mod replica;
mod utils;

//use crate::replica;
//use crate::leader;
use crate::broadcast_channel::BroadcastSender;
use crate::utils::{Operation, Command, Request, Decision, Response};


use crossbeam::channel::{unbounded, Receiver, Sender};
use log::info;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

// contains the handles for control channels for replica and leaders
pub struct SystemHandles {
    client_control_chan_sender: BroadcastSender<client::ControlSignal>,
    replica_control_chan_sender: BroadcastSender<replica::ControlSignal>,
    leader_control_chan_sender: BroadcastSender<leader::ControlSignal>,
    acceptor_control_chan_sender: BroadcastSender<acceptor::ControlSignal>,
}

impl SystemHandles {
    pub fn system_handle_management(
        client_count: usize,
        replica_count: usize,
        leader_count: usize,
        acceptor_count: usize,
    ) -> SystemHandles {
        // get the client control channels for the clients
        let (client_control_chan_sender, client_control_chan_receivers) =
            broadcast_channel::construct::<client::ControlSignal>(client_count.clone() as u8);

        // get the broadcast control channels for the replicas
        let (replica_control_chan_sender, replica_control_chan_receivers) =
            broadcast_channel::construct::<replica::ControlSignal>(replica_count.clone() as u8);

        // get the broadcast control channels for the leaders
        let (leader_control_chan_sender, leader_control_chan_receivers) =
            broadcast_channel::construct::<leader::ControlSignal>(leader_count.clone() as u8);

        // get the broadcast control channels for the acceptors
        let (acceptor_control_chan_sender, acceptor_control_chan_receivers) =
            broadcast_channel::construct::<acceptor::ControlSignal>(acceptor_count.clone() as u8);





        // get the vector of handles and then reverse it
        // reversing eases the assignment of the handles to the clients, replicas, leaders and acceptors by pop later
        let mut split_client_control_chan_receivers = client_control_chan_receivers.handle_split();
        let mut split_replica_control_chan_receivers =
            replica_control_chan_receivers.handle_split();
        let mut split_leader_control_chan_receivers = leader_control_chan_receivers.handle_split();
        let mut split_acceptor_control_chan_receivers =
            acceptor_control_chan_receivers.handle_split();
        split_client_control_chan_receivers.reverse();
        split_replica_control_chan_receivers.reverse();
        split_leader_control_chan_receivers.reverse();
        split_acceptor_control_chan_receivers.reverse();




        // hashmap for collecting all broadcast channel receiver handles for replicas while iterating over clients
        let mut hashmap_client_replica_broadcast_chan_receivers: HashMap<usize, Vec<Receiver<Request>>> =
            HashMap::new();
        // hashmap for collecting all broadcast channel receiver handles for leaders while iterating over replicas
        let mut hashmap_replica_leader_broadcast_chan_receivers: HashMap<usize, Vec<Receiver<u8>>> =
            HashMap::new();
        // hashmap for collecting all broadcast channel receiver handles for acceptors while iterating over leaders
        let mut hashmap_leader_acceptor_broadcast_chan_receivers: HashMap<
            usize,
            Vec<Receiver<u8>>,
        > = HashMap::new();



        // vector for collecting all the sender handles of the mpsc channels from replica to all clients
        // the sender handles of this channel will be cloned to all the replicas
        let mut replica_all_clients_mpsc_chan_senders: Vec<Sender<Response>> = Vec::new();

        // vector for collecting all the sender handles of the mpsc channels from acceptors to all leaders
        // the sender handles of this channel will be cloned to all the replicas
        let mut acceptor_all_leaders_for_commanders_mpsc_chan_senders: Vec<Sender<u8>> = Vec::new();
        let mut acceptor_all_leaders_for_scouts_mpsc_chan_senders: Vec<Sender<u8>> = Vec::new();


        // hashmap for collecting all broadcast channel sender handles for leaders while iterating over leaders
        let mut hashmap_leader_replica_broadcast_chan_senders: HashMap<usize, BroadcastSender<Decision>> =
            HashMap::new();
        // hashmap for collecting all broadcast channel receiver handles for replicas while iterating over leaders
        let mut hashmap_leader_replica_broadcast_chan_receivers: HashMap<usize, Vec<Receiver<Decision>>> =
            HashMap::new();







        // iterating over the leader
        for leader_id in 0..leader_count {
            // get the broadcasts channel from the leader to replicas
            let (leader_replica_broadcast_chan_sender, leader_replica_broadcast_chan_receivers) =
                broadcast_channel::construct::<Decision>(replica_count.clone() as u8);

            // collect the sender handles for the leaders
            hashmap_leader_replica_broadcast_chan_senders
                .insert(leader_id, leader_replica_broadcast_chan_sender);

            // collect the receiver handles for the replicas
            let mut split_leader_replica_broadcast_chan_receivers =
                leader_replica_broadcast_chan_receivers.handle_split();
            split_leader_replica_broadcast_chan_receivers.reverse();
            hashmap_leader_replica_broadcast_chan_receivers
                .insert(leader_id, split_leader_replica_broadcast_chan_receivers);
        }









        // iterate over each client
        for client_id in 0..client_count {
            // get the broadcast channel from current client to all replicas
            let (client_replica_broadcast_chan_sender, client_replica_broadcast_chan_receivers) =
                broadcast_channel::construct::<Request>(replica_count.clone() as u8);

            // get the mpsc channel from replicas to client
            let (replica_client_mpsc_chan_sender, replica_client_mpsc_chan_receiver) = unbounded();
            // inserting the handle for the chanel into the vec, to be used in replica
            replica_all_clients_mpsc_chan_senders.push(replica_client_mpsc_chan_sender);

            // build the client
            let client_context = client::new(
                client_id as u8,
                client_replica_broadcast_chan_sender,
                replica_client_mpsc_chan_receiver,
                split_client_control_chan_receivers.pop().unwrap(),
            );

            // start the client in paused mode
            client_context.start();

            // collect the receiver handles for replicas in the hashmap
            let mut split_client_replica_broadcast_chan_receivers =
                client_replica_broadcast_chan_receivers.handle_split();
            split_client_replica_broadcast_chan_receivers.reverse();
            hashmap_client_replica_broadcast_chan_receivers
                .insert(client_id, split_client_replica_broadcast_chan_receivers);
        }










        // iterate over each replica
        for replica_id in 0..replica_count {
            // collect the receiver handles of the broadcast channels from all the clients
            let mut client_replica_broadcast_chan_receivers: Vec<Receiver<Request>> = Vec::new();
            for client_id in 0..client_count {
                // retrieving the entry corresponding to replica_id
                // this approach taken because HashMap doesn't implement IndexMut trait
                let mut hashmap_entry_client_id = hashmap_client_replica_broadcast_chan_receivers
                    .remove(&client_id)
                    .unwrap();

                // update
                client_replica_broadcast_chan_receivers
                    .push(hashmap_entry_client_id.pop().unwrap());

                // insert back the entry corresponding to replica_id
                hashmap_client_replica_broadcast_chan_receivers
                    .insert(client_id, hashmap_entry_client_id);
            }

            // collect the receiver handles of the broadcast channels from all the leaders
            let mut leader_replica_broadcast_chan_receivers: Vec<Receiver<Decision>> = Vec::new();
            for leader_id in 0..leader_count {
                let mut hashmap_entry_leader_id = hashmap_leader_replica_broadcast_chan_receivers
                    .remove(&leader_id)
                    .unwrap();

                leader_replica_broadcast_chan_receivers
                    .push(hashmap_entry_leader_id.pop().unwrap());

                hashmap_leader_replica_broadcast_chan_receivers
                    .insert(leader_id, hashmap_entry_leader_id);
            }

            // get the broadcast channel from curent replica to all leaders
            let (replica_leader_broadcast_chan_sender, replica_leader_broadcast_chan_receivers) =
                broadcast_channel::construct::<u8>(leader_count.clone() as u8);

            // build the replica
            // do note that one clone of replica_all_clients_mpsc_chan_senders is left unassigned to any replica
            let replica_context = replica::new(
                replica_id as u8,
                client_replica_broadcast_chan_receivers,
                replica_all_clients_mpsc_chan_senders.clone(),
                replica_leader_broadcast_chan_sender,
                leader_replica_broadcast_chan_receivers,
                split_replica_control_chan_receivers.pop().unwrap(),
            );

            // start the replica in paused mode
            replica_context.start();

            // collect the receiver handles for leaders in the hashmap
            let mut split_replica_leader_broadcast_chan_receivers =
                replica_leader_broadcast_chan_receivers.handle_split();
            split_replica_leader_broadcast_chan_receivers.reverse();
            hashmap_replica_leader_broadcast_chan_receivers
                .insert(replica_id, split_replica_leader_broadcast_chan_receivers);
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
                hashmap_replica_leader_broadcast_chan_receivers
                    .insert(replica_id, hashmap_entry_replica_id);
            }

            // get the broadcast channel from curent leader to all acceptors
            let (leader_acceptor_broadcast_chan_sender, leader_acceptor_broadcast_chan_receivers) =
                broadcast_channel::construct::<u8>(acceptor_count.clone() as u8);


            // get the mpsc channel from acceptors to the leaders
            let (acceptor_leader_for_commander_mpsc_chan_sender, acceptor_leader_for_commander_mpsc_chan_receiver) 
                = unbounded();
            let (acceptor_leader_for_scout_mpsc_chan_sender, acceptor_leader_for_scout_mpsc_chan_receiver)
                = unbounded();
            // to be used in the acceptor
            acceptor_all_leaders_for_commanders_mpsc_chan_senders
                .push(acceptor_leader_for_commander_mpsc_chan_sender);
            acceptor_all_leaders_for_scouts_mpsc_chan_senders
                .push(acceptor_leader_for_scout_mpsc_chan_sender);


            // build the leader
            let leader_context = leader::new(
                leader_id as u8,
                replica_leader_broadcast_chan_receivers,
                hashmap_leader_replica_broadcast_chan_senders
                    .remove(&leader_id)
                    .unwrap(),
                leader_acceptor_broadcast_chan_sender,
                acceptor_leader_for_commander_mpsc_chan_receiver,
                acceptor_leader_for_scout_mpsc_chan_receiver,
                split_leader_control_chan_receivers.pop().unwrap(),
            );

            // start the leader in paused mode
            leader_context.start();

            // collect the receiver handles for acceptors in the hashmap
            let mut split_leader_acceptor_broadcast_chan_receivers =
                leader_acceptor_broadcast_chan_receivers.handle_split();
            split_leader_acceptor_broadcast_chan_receivers.reverse();
            hashmap_leader_acceptor_broadcast_chan_receivers
                .insert(leader_id, split_leader_acceptor_broadcast_chan_receivers);
        }













        // iterate over each acceptor
        for acceptor_id in 0..acceptor_count {
            // collect the receiver handles of the broadcast channels from all the leaders
            let mut leader_acceptor_broadcast_chan_receivers: Vec<Receiver<u8>> = Vec::new();

            for leader_id in 0..leader_count {
                // retrieving the entry corresponding to leader_id
                // this approach taken because HashMap doesn't implement IndexMut trait
                let mut hashmap_entry_leader_id = hashmap_leader_acceptor_broadcast_chan_receivers
                    .remove(&leader_id)
                    .unwrap();

                // update
                leader_acceptor_broadcast_chan_receivers
                    .push(hashmap_entry_leader_id.pop().unwrap());

                // insert back the entry corresponding to leader_id
                hashmap_leader_acceptor_broadcast_chan_receivers
                    .insert(leader_id, hashmap_entry_leader_id);
            }

            // build the acceptor
            let acceptor_context = acceptor::new(
                acceptor_id as u8,
                leader_acceptor_broadcast_chan_receivers,
                acceptor_all_leaders_for_commanders_mpsc_chan_senders.clone(),
                acceptor_all_leaders_for_scouts_mpsc_chan_senders.clone(),
                split_acceptor_control_chan_receivers.pop().unwrap(),
            );

            // start the acceptor in paused mode
            acceptor_context.start();
        }

        SystemHandles {
            client_control_chan_sender,
            replica_control_chan_sender,
            leader_control_chan_sender,
            acceptor_control_chan_sender,
        }
    }

    pub fn activate_broadcast(
        &self,
        num_broadcasts: u8,
        client_count: u8,
        replica_count: u8,
        leader_count: u8,
    ) {
        // activating clients, replicas, leaders and acceptors; broadcast will start now
        self.client_control_chan_sender
            .send(client::ControlSignal::Run(num_broadcasts));
        self.replica_control_chan_sender
            .send(replica::ControlSignal::Run(client_count * num_broadcasts));
        self.leader_control_chan_sender
            .send(leader::ControlSignal::Run(
                replica_count * client_count * num_broadcasts,
            ));
        self.acceptor_control_chan_sender
            .send(acceptor::ControlSignal::Run(
                leader_count * replica_count * client_count * num_broadcasts,
            ));

        thread::sleep(Duration::from_secs(5));
        println!(
            "Exit signal sent; hopefully, clients, leaders, replicas, acceptors in paused mode"
        );
        self.client_control_chan_sender
            .send(client::ControlSignal::Exit);
        self.replica_control_chan_sender
            .send(replica::ControlSignal::Exit);
        self.leader_control_chan_sender
            .send(leader::ControlSignal::Exit);
        self.acceptor_control_chan_sender
            .send(acceptor::ControlSignal::Exit);
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let client_count = 2 as usize;
        let replica_count = 3 as usize;
        let leader_count = 3 as usize;
        let acceptor_count = 3 as usize;
        let num_msgs = 4u8;
        let system_handles = SystemHandles::system_handle_management(
            client_count,
            replica_count,
            leader_count,
            acceptor_count,
        );
        system_handles.activate_broadcast(
            num_msgs,
            client_count as u8,
            replica_count as u8,
            leader_count as u8,
        );
        assert_eq!(2 + 2, 4);
    }
}
