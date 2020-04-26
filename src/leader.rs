use std::thread;
use crossbeam::channel::{unbounded, Sender, Receiver, TryRecvError, RecvError};


enum OperatingState {
    Paused,
    Run(u8),
    Exit,
} 



pub enum ControlSignal {
    Paused,
    Run(u8),
    Exit,
}




pub struct Context {
    id: u8,
    broadcast_chan_receiver: Receiver<u8>,
    control_chan_receiver: Receiver<ControlSignal>,
    operating_state: OperatingState,
}




pub fn new(
    id: u8, 
    broadcast_chan_receiver: Receiver<u8>,
    control_chan_receiver: Receiver<ControlSignal>
) -> Context {

    let context = Context{
        id,
        broadcast_chan_receiver,
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
                            let signal = self.control_chan_receiver.recv().unwrap();
                            self.handle_control_signal(signal);
                        }



                        
                        OperatingState::Run(num_msgs) => {
                            match self.control_chan_receiver.try_recv() {
                                Ok(signal) => {
                                    println!("Not handled properly yet !!!!");
                                    self.handle_control_signal(signal);
                                }
                                Err(TryRecvError::Empty) => {
                                    // continue to try to receive the broadcast message from replica
                                    if num <= num_msgs {
                                        // need blocking so that num increments appropriately
                                        println!("The received number via broadcast is {}", 
                                                self.broadcast_chan_receiver.recv().unwrap()
                                            );
                                        println!("Number of messages in channel queue: {}", 
                                                self.broadcast_chan_receiver.len()
                                            );
                                        num += 1;
                                    } else {
                                        println!("Leader {} Going into paused state", self.id);
                                        self.operating_state = OperatingState::Paused;
                                    }

                                }
                                Err(TryRecvError::Disconnected) => panic!("Leader {} control channel detached", self.id)
                            }
                        }




                        OperatingState::Exit => {
                            println!("Leader {} Exiting", self.id);
                            break;
                        }

                    }
                }
            })
            .unwrap();
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Paused => {
                self.operating_state = OperatingState::Paused;
            }


            ControlSignal::Run(num_msgs) => {
                self.operating_state = OperatingState::Run(num_msgs);
            }

            ControlSignal::Exit => {
                println!("Exit signal at Leader {} received", self.id);
                self.operating_state = OperatingState::Exit;
            }
        }
    }

}