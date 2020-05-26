use crossbeam::channel::{Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::broadcast_channel::BroadcastSender;
use crate::utils::{Operation, Command, Request, Response};

enum OperatingState {
    Paused,
    Run(u32),
    Exit,
}

#[derive(Clone)]
pub enum ControlSignal {
    Paused,
    Run(u32),
    Exit,
}

pub struct Context {
    // ID of the client
    id: u32,

    // handle to send broadcast message to replica
    client_replica_broadcast_chan_sender: BroadcastSender<Request>,

    // handle to the receiver handle for mpsc channel from all replicas
    replica_client_mpsc_chan_receiver: Receiver<Response>,

    // handle to receive control signals
    control_chan_receiver: Receiver<ControlSignal>,

    // state of the replica
    operating_state: OperatingState,

    // response commands IDs
    response_command_ids: Vec<u32>,
}

pub fn new(
    id: u32,
    client_replica_broadcast_chan_sender: BroadcastSender<Request>,
    replica_client_mpsc_chan_receiver: Receiver<Response>,
    control_chan_receiver: Receiver<ControlSignal>,
) -> Context {
    let context = Context {
        id,
        client_replica_broadcast_chan_sender,
        replica_client_mpsc_chan_receiver,
        control_chan_receiver,
        operating_state: OperatingState::Paused,
        response_command_ids: Vec::new(),
    };

    context
}



impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .spawn(move || {
                let mut num_commands = 1u32;
                let mut num_responses = 1u32;

                loop {
                                       
                    match self.operating_state {
                        OperatingState::Paused => {
                            // println!("Client {} in paused mode", self.id);
                            let signal = self.control_chan_receiver.recv().unwrap();
                            // transition in operating state
                            self.handle_control_signal(signal);
                        }

                        OperatingState::Run(num_msgs) => {
                            // pattern matching the control channel messages
                            match self.control_chan_receiver.try_recv() {
                                Ok(signal) => {
                                    println!("Client {} Not handled properly yet! Increase the grace time period!", self.id);
                                    // transition in operating state
                                    self.handle_control_signal(signal);
                                }
                                Err(TryRecvError::Empty) => {
                                    if num_commands <= num_msgs {
                                        self.send_broadcast_message(num_commands.clone());
                                        num_commands += 1;
                                    }
                                    
                                    if num_responses <= num_msgs {
                                        num_responses = self.processing_response_message(num_responses.clone());
                                    } else {
                                        // println!("Client {} Going into paused state", self.id);
                                        self.operating_state = OperatingState::Paused;
                                    }
                                }
                                Err(TryRecvError::Disconnected) => {
                                    panic!("Client control channel detached")
                                }
                            };
                        }

                        OperatingState::Exit => {
                            println!("Client {} deactivated.......................", self.id);
                            break;
                        }
                    }

                    thread::sleep(Duration::from_nanos(100));
                }
            })
            .unwrap();
    }



    fn send_broadcast_message(&self, num: u32) {
        let mut operation = Operation::Null;

        match num%4 {
            0 => { operation = Operation::Add(1i32); }
            1 => { operation = Operation::Subtract(1i32); }
            2 => { operation = Operation::Multiply(2i32); }
            _ => { operation = Operation::Add(1i32); }
        }

        let command = Command::create( self.id.clone(), num, operation);

        self.client_replica_broadcast_chan_sender
            .send(Request::create(command));
    }





    // function for handling response message coming from the replicas
    fn processing_response_message(&mut self, mut num_responses: u32) -> u32 {

        if let Ok(response) = &self.replica_client_mpsc_chan_receiver.try_recv() {
            if self.response_command_ids.contains(&response.get_command_id()) == false {
                println!("Result for command with command ID {} at client {} is: {}", 
                        response.get_command_id(), 
                        self.id,
                        response.get_result());

                // update num_responses only it is a new command
                self.response_command_ids.push(response.get_command_id().clone());
                num_responses += 1;
            }
        }

        num_responses
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
                // println!("Exit signal at Client {} received", self.id);
                self.operating_state = OperatingState::Exit;
            }
        }
    }
}
