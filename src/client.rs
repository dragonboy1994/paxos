use crossbeam::channel::{Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::broadcast_channel::BroadcastSender;
use crate::utils::{Operation, Command, Request};

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
    client_replica_broadcast_chan_sender: BroadcastSender<Request>,

    // handle to the receiver handle for mpsc channel from all replicas
    replica_client_mpsc_chan_receiver: Receiver<u8>,

    // handle to receive control signals
    control_chan_receiver: Receiver<ControlSignal>,

    // state of the replica
    operating_state: OperatingState,
}

pub fn new(
    id: u8,
    client_replica_broadcast_chan_sender: BroadcastSender<Request>,
    replica_client_mpsc_chan_receiver: Receiver<u8>,
    control_chan_receiver: Receiver<ControlSignal>,
) -> Context {
    let context = Context {
        id,
        client_replica_broadcast_chan_sender,
        replica_client_mpsc_chan_receiver,
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
                                        self.send_broadcast_message(num.clone());
                                        num += 1;
                                    } else {
                                        println!("Client {} Going into paused state", self.id);
                                        self.operating_state = OperatingState::Paused;
                                    }
                                }
                                Err(TryRecvError::Disconnected) => {
                                    panic!("Client control channel detached")
                                }
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

    fn send_broadcast_message(&self, num: u8) {
        let mut operation = Operation::Null;

        match num%4 {
            0 => { operation = Operation::Add(1f64); }
            1 => { operation = Operation::Subtract(1f64); }
            2 => { operation = Operation::Multiply(2f64); }
            3 => { operation = Operation::Divide(2f64); }
            _ => {println!("This is incomplete!!!! Need makeover");}
        }

        let command = Command::create( self.id.clone(), num, operation);
        println!("Command is {:#?}", command);

        println!("Client {} has broadcast a command", self.id);
        self.client_replica_broadcast_chan_sender
            .send(Request::create(command));
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
