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
    // all messages received in broadcast from replica
    // messages: Vec<u8>

    // handle to send broadcast messages
    replica_leader_broadcast_chan_sender: BroadcastSender<u8>,

    // handle to receive contral signals
    control_chan_receiver: Receiver<ControlSignal>,

    // state of the replica
    operating_state: OperatingState,
}



pub fn new(
    replica_leader_broadcast_chan_sender: BroadcastSender<u8>
) -> (Context, Sender<ControlSignal>) {
    
    let (control_chan_sender, control_chan_receiver) = unbounded();

    let context = Context{
        replica_leader_broadcast_chan_sender,
        control_chan_receiver,
        operating_state: OperatingState::Paused,
    };

    (context, control_chan_sender)
}




impl Context {

    pub fn start(mut self) {
        thread::Builder::new()
            .spawn(move || {
                let mut num = 1u8;

                loop {
                    match self.operating_state {


                        OperatingState::Paused => {
                            let signal = self.control_chan_receiver.recv().unwrap();
                            // transition in operating state
                            self.handle_control_signal(signal);
                        }


                        OperatingState::Run(num_msgs) => {
                            // send message to the receiver
                            match self.control_chan_receiver.try_recv() {
                                Ok(signal) => {    
                                    // transition in operating state                                
                                    self.handle_control_signal(signal);
                                }
                                Err(TryRecvError::Empty) => {
                                    if num <= num_msgs {
                                        self.processing_broadcast_message_from_client();
                                        num += 1;
                                    } else {
                                        println!("Replica Going into paused state");
                                        self.operating_state = OperatingState::Paused;
                                    }
                                    
                                }
                                Err(TryRecvError::Disconnected) => panic!("Replica control channel detached")
                            };
                        }


                        OperatingState::Exit => {
                            println!("Replica exiting gracefully");
                            break;
                        } 
                        
                    }

                    thread::sleep(Duration::from_nanos(100));
                }

            })
            .unwrap();
    }





    fn processing_broadcast_message_from_client(&self) {
        self.replica_leader_broadcast_chan_sender.send(99);
    }






    fn handle_control_signal(&mut self, signal: ControlSignal) {
        // change the operating state 
        match signal {
            ControlSignal::Paused => {
                self.operating_state = OperatingState::Paused;
            }

            ControlSignal::Run(num_msgs) => {
                println!("Replica activated!");
                self.operating_state = OperatingState::Run(num_msgs);
            }

            ControlSignal::Exit => {
                println!("Exit signal at Replica received");
                self.operating_state = OperatingState::Exit;
            }
        }
    }
}