use std::thread;
use crossbeam::channel::{Receiver, TryRecvError};


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

    // all messages received in broadcast from replica
    messages: Vec<u8>,

    // handle for the broadcast channel between all replicas and the leader
    replica_leader_broadcast_chan_receiver: Vec<Receiver<u8>>,

    // handle for controlling the leader operating state
    control_chan_receiver: Receiver<ControlSignal>,

    // operating state of the leader
    operating_state: OperatingState,
}




pub fn new(
    id: u8, 
    replica_leader_broadcast_chan_receiver: Vec<Receiver<u8>>,
    control_chan_receiver: Receiver<ControlSignal>
) -> Context {

    let context = Context{
        id,
        messages: Vec::new(),
        replica_leader_broadcast_chan_receiver,
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
                            let signal = self.control_chan_receiver.recv().unwrap();
                            self.handle_control_signal(signal);
                        }



                        
                        OperatingState::Run(num_msgs) => {

                            // analyzing under various control channel state
                            match self.control_chan_receiver.try_recv() {
                                
                                // some signal from replica arrived
                                Ok(signal) => {
                                    println!("Not handled properly yet !!!!");
                                    self.handle_control_signal(signal);
                                }

                                // empty control channel, feel free to continue interacting with the replicas
                                Err(TryRecvError::Empty) => {
                                    if self.messages.len() < num_msgs as usize {
                                        self.processing_broadcast_message_from_replica();
                                    } else {
                                        println!("Leader {} Going into paused state", self.id);
                                        self.operating_state = OperatingState::Paused;
                                    }
                                }


                                // Disconnected control channel
                                Err(TryRecvError::Disconnected) => panic!("Leader {} control channel detached", self.id)
                            }
                        }




                        OperatingState::Exit => {
                            println!("Leader {} exiting gracefully", self.id);
                            break;
                        }

                    }
                }
            })
            .unwrap();
    }



    // processing of the received messages
    fn processing_broadcast_message_from_replica(&mut self) {
         

        // iterate over the receiver handles from all the replicas to scan for any possible messages
        for handle in &self.replica_leader_broadcast_chan_receiver {

            // using try_recv() so that leader is free to do broadcast of its own to acceptors
            // non-blocking from broadcast of replica desirer
            match handle.try_recv() {

                Ok(message) => {
                    println!("The received message at leader {} is {}", 
                            self.id,
                            message
                            );
                    self.messages.push(message)
                }

                _ => {}

            }
        }
    }




    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Paused => {
                self.operating_state = OperatingState::Paused;
            }


            ControlSignal::Run(num_msgs) => {
                println!("Leader {} activated", self.id);
                self.operating_state = OperatingState::Run(num_msgs);
            }

            ControlSignal::Exit => {
                println!("Exit signal at Leader {} received", self.id);
                self.operating_state = OperatingState::Exit;
            }
        }
    }

}