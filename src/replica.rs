use crossbeam::channel::{Receiver, Sender, TryRecvError};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;


use crate::broadcast_channel::BroadcastSender;
use crate::utils::{Operation, Command, Request, Decision, Response, Propose};


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



    // for broadcast mechanism
    // collecting all messages received in broadcast at replica from client
    // push new messages from back, pop old messages from front
    // this is a dummy variable -> remove later
    messages_from_client: VecDeque<u8>,

    // handle for the broadcast channel between all clients and the replica
    client_replica_broadcast_chan_receiver: Vec<Receiver<Request>>,

    // vec of handle for the mpsc channels from the replica to all the clients
    replica_all_clients_mpsc_chan_senders: Vec<Sender<Response>>,

    // handle to send broadcast messages to the leaders
    replica_leader_broadcast_chan_sender: BroadcastSender<Propose>,

    // handle for the receiver side of broadcast channel between all leaders and the replica
    leader_replica_broadcast_chan_receiver: Vec<Receiver<Decision>>,

    // handle to receive contral signals
    control_chan_receiver: Receiver<ControlSignal>,

    // operation state of the replica
    operating_state: OperatingState,



    // for consensus mechanism
    // all taken from the PMMC paper
    // application state
    state: i32, 

    // index of the next slot in replica has not proposed any command yet
    slot_in: u8,

    // index of the next slot for which decision has to be leanred before it can update application state
    slot_out: u8,

    // set of requests that replica hasn't proposed or decided yet
    requests: VecDeque<Command>,

    // set of proposals that are currently outstanding
    proposals: HashMap<u8, Command>,

    // set of proposals that are known to have been decided
    decisions: HashMap<u8, Command>,

    // skipping the leaders for now
    //static configuration

}

pub fn new(
    id: u8,
    client_replica_broadcast_chan_receiver: Vec<Receiver<Request>>,
    replica_all_clients_mpsc_chan_senders: Vec<Sender<Response>>,
    replica_leader_broadcast_chan_sender: BroadcastSender<Propose>,
    leader_replica_broadcast_chan_receiver: Vec<Receiver<Decision>>,
    control_chan_receiver: Receiver<ControlSignal>,
) -> Context {
    let context = Context {
        id,
        messages_from_client: VecDeque::new(),
        client_replica_broadcast_chan_receiver,
        replica_all_clients_mpsc_chan_senders,
        replica_leader_broadcast_chan_sender,
        leader_replica_broadcast_chan_receiver,
        control_chan_receiver,
        operating_state: OperatingState::Paused,
        state: 0i32,
        slot_in: 1u8,
        slot_out: 1u8,
        requests: VecDeque::new(),
        proposals: HashMap::new(),
        decisions: HashMap::new()
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
                                    if self.slot_out <= num_msgs {
                                        self.processing_broadcast_message_from_client();

                                        // uncomment them later
                                        self.processing_decision_message_from_leader();
                                        self.propose();
                                    } else {
                                        println!("Replica {} Going into paused state", self.id);
                                        println!("Decision list at replica {} is {:#?}", self.id, self.decisions);
                                        println!("The state at replica {} is {}", self.id, self.state);
                                        self.operating_state = OperatingState::Paused;
                                    }
                                }
                                Err(TryRecvError::Disconnected) => {
                                    panic!("Replica control channel detached")
                                }
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
                    // println!("The received message at replica {} is {:#?}", self.id, message);
                    // push into the requests 
                    self.requests.push_back(message.clone().get_command());
                }

                _ => {}
            }
        }

    }



    fn processing_decision_message_from_leader(&mut self) {
        // process the decision messages received from the leader
        for handle in &self.leader_replica_broadcast_chan_receiver {
            match handle.try_recv() {
                Ok(message) => {
                    let (command, slot) = message.get_details();
                    self.decisions.insert(slot, command);
                    // println!("Decision message inserted");
                    


                    while self.decisions.contains_key(&self.slot_out) == true {
                        let command_prime = self.decisions.get(&self.slot_out).unwrap().clone();
                        if self.proposals.contains_key(&self.slot_out) == true {
                            // removed from proposals
                            let command_prime_prime = self.proposals.remove(&self.slot_out).unwrap();
                            if command_prime_prime != command_prime {
                                self.requests.push_back(command_prime_prime);
                            }
                        }
                        self.state = self.perform(command_prime);
                        // slot_out increment outside because of error with mutable and immutable
                        // we will come back to it later
                        self.slot_out += 1;
                    }
                }

                _ => {}
            }


        }
    }



    // for checking whether a command is in the decision
    // I think we can do better
    fn decision_contains_command(&self, command: Command) -> bool {
        for slot in self.decisions.keys() {
            if (self.decisions.get(slot).unwrap().clone() == command) && (slot.clone() < self.slot_out) {
                return true;
            } 
        }
        return false;
    }


    fn perform(&self, command: Command)-> i32 {

        let mut next = 0i32;
        let mut result = 0i32;  

        // skipping the true case as it only involves slot_out increment only
        // increment done outside
        if self.decision_contains_command(command.clone()) == false {
            // getting updated state
            // state and result same for our case -> bit unclear
            match command.get_operation() {
                Operation::Add(x) => {
                    next = self.state.clone() + x;
                    result = self.state.clone() + x;
                }

                Operation::Subtract(y) => {
                    next = self.state.clone() - y;
                    result = self.state.clone() - y;

                }

                Operation::Multiply(z) => {
                    next = self.state.clone() * z;
                    result = self.state.clone() * z;
                }

                _ => {}
            }
        }

        // send response to the client
        // println!("Replica {} has sent reponse", self.id);
        self.replica_all_clients_mpsc_chan_senders[command.get_client_id() as usize]
            .send(Response::create(command.get_command_id(), result));

        next


    }

   

    fn propose(&mut self) {
        // if requests is not empty
        while self.requests.is_empty() == false {
            // skipped first as there is no reconfig operation
            if self.decisions.contains_key(&self.slot_in) == false {
                let command = self.requests.pop_front().unwrap();
                self.proposals.insert(self.slot_in.clone(), command.clone());
                // uncomment the following line later
                // broadcast to leaders
                // println!("Replica {} has broadcast propose message", self.id);
                self.replica_leader_broadcast_chan_sender.send(Propose::create(self.slot_in.clone(), command));
            }
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
