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

    // handle for the broadcast channel between all leaders and the acceptor
    leader_acceptor_broadcast_chan_receiver: Vec<Receiver<u8>>,

    // handle for controlling the leader operating state
    control_chan_receiver: Receiver<ControlSignal>,

    // operating state of the leader
    operating_state: OperatingState,
}




pub fn new(
    id: u8, 
    leader_acceptor_broadcast_chan_receiver: Vec<Receiver<u8>>,
    control_chan_receiver: Receiver<ControlSignal>
) -> Context {

    let context = Context{
        id,
        messages: Vec::new(),
        leader_acceptor_broadcast_chan_receiver,
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
                            println!("Acceptor {} in paused mode", self.id);
                            let signal = self.control_chan_receiver.recv().unwrap();
                            self.control_signal_processing(signal);
                        }



                        
                        OperatingState::Run(num_msgs) => {

                            // analyzing under various control channel state
                            match self.control_chan_receiver.try_recv() {
                                
                                // some control arrived
                                Ok(signal) => {
                                    println!("Acceptor {} going premature exit !!!!", self.id);
                                    self.control_signal_processing(signal);
                                }

                                // empty control channel, feel free to continue interacting with the replicas
                                Err(TryRecvError::Empty) => {
                                    if self.messages.len() < num_msgs as usize {
                                        self.message_processing();
                                    } else {
                                        println!("Acceptor {} going into paused state", self.id);
                                        self.operating_state = OperatingState::Paused;
                                    }
                                }


                                // Disconnected control channel
                                Err(TryRecvError::Disconnected) => panic!("Acceptor {} control channel detached", self.id)
                            }
                        }




                        OperatingState::Exit => {
                            println!("Acceptor {} exiting gracefully", self.id);
                            break;
                        }

                    }
                }
            })
            .unwrap();
    }
   
    // processing of the received messages
    fn message_processing(&mut self) {
         
        // iterate over the receiver handles from all the leaders to scan for any possible messages
        for handle in &self.leader_acceptor_broadcast_chan_receiver {

            // using try_recv() so that acceptor is free to do other activities
            match handle.try_recv() {

                Ok(message) => {
                    println!("The received message at acceptor {} is {}", 
                            self.id,
                            message
                            );
                    self.messages.push(message)
                }

                _ => {}

            }
        }
    }


    fn control_signal_processing(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Paused => {
                self.operating_state = OperatingState::Paused;
            }


            ControlSignal::Run(num_msgs) => {
                println!("Acceptor {} activated", self.id);
                self.operating_state = OperatingState::Run(num_msgs);
            }

            ControlSignal::Exit => {
                println!("Exit signal at Acceptor {} received", self.id);
                self.operating_state = OperatingState::Exit;
            }
        }
    }    


}