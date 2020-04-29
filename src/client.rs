use std::thread;
use crossbeam::channel::{Receiver, TryRecvError};
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

    // ID of the client
    id: u8,

    // handle to send broadcast message to replica
    client_replica_broadcast_chan_sender: BroadcastSender<u8>,

    // handle to receive control signals
    control_chan_receiver: Receiver<ControlSignal>,

    // state of the replica
    operating_state: OperatingState,    
}




pub fn new(
    id: u8,
    client_replica_broadcast_chan_sender: BroadcastSender<u8>,
    control_chan_receiver: Receiver<ControlSignal>
) -> Context {
    
    let context = Context{
        id,
        client_replica_broadcast_chan_sender,
        control_chan_receiver,
        operating_state: OperatingState::Paused,
    };

    context
}




impl Context {

    pub fn start(mut self) {
        thread::Builder::new()
            .spawn(move || {
                let mut num = 1u8;

                loop {
                    match self.operating_state {


                        OperatingState::Paused => {
                            println!("Client {} in paused mode", self.id);
                            let signal = self.control_chan_receiver.recv().unwrap();
                            // transition in operating state
                            self.handle_control_signal(signal);
                        }


                        OperatingState::Run(num_msgs) => {
                            // pattern matching the control channel messages 
                            match self.control_chan_receiver.try_recv() {
                                Ok(signal) => {    
                                    println!("Client {} Not handled properly yet !!!!", self.id);
                                    // transition in operating state                                
                                    self.handle_control_signal(signal);
                                }
                                Err(TryRecvError::Empty) => {
                                    if num <= num_msgs {
                                        self.send_broadcast_message();
                                        num += 1;
                                    } else {
                                        println!("Client {} Going into paused state", self.id);
                                        self.operating_state = OperatingState::Paused;
                                    }
                                    
                                }
                                Err(TryRecvError::Disconnected) => panic!("Client control channel detached")
                            };
                        }


                        OperatingState::Exit => {
                            println!("Client {} exiting gracefully", self.id);
                            break;
                        } 
                        
                    }

                    thread::sleep(Duration::from_nanos(100));
                }

            })
            .unwrap();
    }



    fn send_broadcast_message(&self) {
        println!("Client {} has broadcast", self.id);
        self.client_replica_broadcast_chan_sender.send(self.id.clone());
    }




    fn handle_control_signal(&mut self, signal: ControlSignal) {
        // change the operating state 
        match signal {
            ControlSignal::Paused => {
                self.operating_state = OperatingState::Paused;
            }

            ControlSignal::Run(num_msgs) => {
                println!("Client {} activated!", self.id);
                self.operating_state = OperatingState::Run(num_msgs);
            }

            ControlSignal::Exit => {
                println!("Exit signal at Client {} received", self.id);
                self.operating_state = OperatingState::Exit;
            }
        }
    }



}


