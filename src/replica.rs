use std::thread;
use crossbeam::channel::{Receiver, TryRecvError};
use std::time::Duration;
use std::collections::VecDeque;

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
    // ID of the leader
    id: u8,

    // collecting all messages received in broadcast at replica from client
    // push new messages from back, pop old messages from front
    messages_from_client: VecDeque<u8>,

    // add details later
    slot_in: u8,

    // handle for the broadcast channel between all clients and the replica
    client_replica_broadcast_chan_receiver: Vec<Receiver<u8>>,

    // handle to send broadcast messages
    replica_leader_broadcast_chan_sender: BroadcastSender<u8>,

    // handle to receive contral signals
    control_chan_receiver: Receiver<ControlSignal>,

    // state of the replica
    operating_state: OperatingState,
}



pub fn new(
    id: u8,
    client_replica_broadcast_chan_receiver: Vec<Receiver<u8>>,
    replica_leader_broadcast_chan_sender: BroadcastSender<u8>,
    control_chan_receiver: Receiver<ControlSignal>
) -> Context {
    
    let context = Context{
        id,
        messages_from_client: VecDeque::new(),
        slot_in: 1u8,
        client_replica_broadcast_chan_receiver,
        replica_leader_broadcast_chan_sender,
        control_chan_receiver,
        operating_state: OperatingState::Paused,
    };

    context
}




impl Context {

    pub fn start(mut self) {
        thread::Builder::new()
            .spawn(move || {


                loop {
                    match self.operating_state {


                        OperatingState::Paused => {
                            println!("Replica {} in paused mode", self.id);
                            let signal = self.control_chan_receiver.recv().unwrap();
                            // transition in operating state
                            self.handle_control_signal(signal);
                        }


                        OperatingState::Run(num_msgs) => {
                            // send message to the receiver
                            match self.control_chan_receiver.try_recv() {
                                Ok(signal) => {    
                                    // transition in operating state   
                                    println!("Replica {} Not handled properly yet !!!!", self.id);                             
                                    self.handle_control_signal(signal);
                                }
                                Err(TryRecvError::Empty) => {
                                    if self.slot_in <= num_msgs {
                                        self.processing_broadcast_message_from_client();
                                    } else {
                                        println!("Replica {} Going into paused state", self.id);
                                        self.operating_state = OperatingState::Paused;
                                    }
                                    
                                }
                                Err(TryRecvError::Disconnected) => panic!("Replica control channel detached")
                            };
                        }


                        OperatingState::Exit => {
                            println!("Replica {} exiting gracefully", self.id);
                            break;
                        } 
                        
                    }

                    thread::sleep(Duration::from_nanos(100));
                }

            })
            .unwrap();
    }




    // receiving and processing of the messages received from clients
    // sending broadcast messages to the leaders
    fn processing_broadcast_message_from_client(&mut self) {

        // process the messages received from the clients 
        // iterate over the receiver handles from all the clients to scan for any possible messages
        for handle in &self.client_replica_broadcast_chan_receiver {

            // using try_recv() so that we have non-blocking operation for replica
            match handle.try_recv() {

                // received a new message from client
                Ok(message) => {
                    println!("The received message at replica {} is {}",
                            self.id,
                            message
                            );
                    self.messages_from_client.push_back(message);
                }

                _ => {}

            }

        }


        // send broadcast messages to leaders only if there is message from client
        if self.messages_from_client.is_empty() == false {
            self.replica_leader_broadcast_chan_sender
                .send(self.messages_from_client.pop_front().unwrap());
            // go into paused mode only after sending all stiplated messages
            self.slot_in += 1; 
        }
    }






    fn handle_control_signal(&mut self, signal: ControlSignal) {
        // change the operating state 
        match signal {
            ControlSignal::Paused => {
                self.operating_state = OperatingState::Paused;
            }

            ControlSignal::Run(num_msgs) => {
                println!("Replica {} activated!", self.id);
                self.operating_state = OperatingState::Run(num_msgs);
            }

            ControlSignal::Exit => {
                println!("Exit signal at Replica {} received", self.id);
                self.operating_state = OperatingState::Exit;
            }
        }
    }
}