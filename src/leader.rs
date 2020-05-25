use crossbeam::channel::{unbounded, Sender, Receiver, TryRecvError};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use crate::broadcast_channel::BroadcastSender;
use crate::utils::{Operation, Command, Decision, Ballot, P1a, P1b, P2a, P2b, Adopted, Preempted, ScoutMessage};
use crate::scout;
use crate::commander;

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

    // maximum scout ID's assigned till now
    scout_id: u16,

    // maximum commander ID's assigned till now
    commander_id: u16,

    // number of acceptors
    num_acceptors: u8,

    // all messages received in broadcast from replica
    messages_from_replica: VecDeque<u8>,

    // add details later
    slot_in: u8,

    // ballot num 
    ballot_num: Ballot, 

    // boolean flag
    active: bool,

    // a map of slot numbers to proposed commands
    proposals: HashMap<u8, Command>,

    // handle for the broadcast channel between all replicas and the leader
    replica_leader_broadcast_chan_receiver: Vec<Receiver<u8>>,

    // handle to send broadcast messages to replicas
    // this will go to the commander
    leader_replica_broadcast_chan_sender: BroadcastSender<Decision>,

    // handle to send broadcast messages to acceptors
    leader_acceptor_broadcast_chan_sender: BroadcastSender<u8>,

    // handle to send broadcast messages from scouts to acceptors
    scout_acceptor_broadcast_chan_sender: BroadcastSender<P1a>,

    // handle to send broadcast messages from commanders to acceptors
    commander_acceptor_broadcast_chan_sender: BroadcastSender<P2a>,

    // receiving handle for the mpsc channel to commander from all the acceptors
    acceptor_leader_for_commander_mpsc_chan_receiver: Receiver<P2b>,

    // receiving handle for the mpsc channel to scout from all the acceptors
    acceptor_leader_for_scout_mpsc_chan_receiver: Receiver<P1b>,

    // handle for controlling the leader operating state
    control_chan_receiver: Receiver<ControlSignal>,

    // operating state of the leader
    operating_state: OperatingState,

    // sending handles of the channels from the leader to the scouts
    // for sending P1b
    leader_to_all_scouts_sender: Vec<Sender<P1b>>,

    // receive handles of the channels from the scouts to the leader
    // the channel will  be shared between all the scouts
    all_scouts_leader_receiver: Receiver<ScoutMessage>,
    // clone of this sender handle will be shared with all scouts
    all_scouts_leader_sender: Sender<ScoutMessage>,

    // sending handles of the channels from the leader to the commanders
    leader_to_all_commanders_sender: Vec<Sender<P2b>>,

    // receive handles of the channels from the commanders to the leader
    // the channel will  be shared between all the commanders
    all_commanders_leader_receiver: Receiver<Preempted>,
    // clone of this sender handle will be shared with all commanders
    all_commanders_leader_sender: Sender<Preempted>,

}

pub fn new(
    id: u8,
    num_acceptors: u8,
    replica_leader_broadcast_chan_receiver: Vec<Receiver<u8>>,
    leader_replica_broadcast_chan_sender: BroadcastSender<Decision>,
    leader_acceptor_broadcast_chan_sender: BroadcastSender<u8>,
    scout_acceptor_broadcast_chan_sender: BroadcastSender<P1a>,
    commander_acceptor_broadcast_chan_sender: BroadcastSender<P2a>,
    acceptor_leader_for_commander_mpsc_chan_receiver: Receiver<P2b>,
    acceptor_leader_for_scout_mpsc_chan_receiver: Receiver<P1b>,
    control_chan_receiver: Receiver<ControlSignal>,
) -> Context {
    let (all_scouts_leader_sender, all_scouts_leader_receiver) = unbounded();
    let (all_commanders_leader_sender, all_commanders_leader_receiver) = unbounded();

    let context = Context {
        id,
        num_acceptors,
        scout_id: 0u16,
        commander_id: 0u16,
        messages_from_replica: VecDeque::new(),
        slot_in: 1u8,
        ballot_num: Ballot::create(id),
        active: false,
        proposals: HashMap::new(),
        replica_leader_broadcast_chan_receiver,
        leader_replica_broadcast_chan_sender,
        leader_acceptor_broadcast_chan_sender,
        scout_acceptor_broadcast_chan_sender,
        commander_acceptor_broadcast_chan_sender,
        acceptor_leader_for_commander_mpsc_chan_receiver,
        acceptor_leader_for_scout_mpsc_chan_receiver,
        control_chan_receiver,
        operating_state: OperatingState::Paused,
        leader_to_all_scouts_sender: Vec::new(),
        leader_to_all_commanders_sender: Vec::new(),
        all_scouts_leader_receiver,
        all_scouts_leader_sender,
        all_commanders_leader_receiver,
        all_commanders_leader_sender,
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
                            println!("Leader {} in paused mode", self.id);
                            let signal = self.control_chan_receiver.recv().unwrap();
                            self.handle_control_signal(signal);
                        }

                        OperatingState::Run(num_msgs) => {
                            // analyzing under various control channel state
                            match self.control_chan_receiver.try_recv() {
                                // some control arrived
                                Ok(signal) => {
                                    println!("Leader {} Not handled properly yet !!!!", self.id);
                                    self.handle_control_signal(signal);
                                }

                                // empty control channel, feel free to continue interacting with the replicas
                                Err(TryRecvError::Empty) => {
                                    if self.slot_in <= num_msgs {
                                        self.processing_broadcast_message_from_replica();
                                    } else {
                                        println!("Leader {} Going into paused state", self.id);
                                        self.operating_state = OperatingState::Paused;
                                    }
                                }

                                // Disconnected control channel
                                Err(TryRecvError::Disconnected) => {
                                    panic!("Leader {} control channel detached", self.id)
                                }
                            }
                        }

                        OperatingState::Exit => {
                            println!("Leader {} exiting gracefully", self.id);
                            break;
                        }
                    }

                    thread::sleep(Duration::from_nanos(100));
                }
            })
            .unwrap();
    }





    // has to eventually remove this part
    // processing of the received messages
    fn processing_broadcast_message_from_replica(&mut self) {
        // iterate over the receiver handles from all the replicas to scan for any possible messages
        for handle in &self.replica_leader_broadcast_chan_receiver {
            // using try_recv() so that leader is free to do broadcast of its own to acceptors
            // non-blocking from broadcast of replica desirer
            match handle.try_recv() {
                Ok(message) => {
                    println!("The received message at leader {} is {}", self.id, message);
                    self.messages_from_replica.push_back(message);
                }
                
                _ => {}
            }
        }

        // send broadcast messages to acceptors only if there is message from client
        if self.messages_from_replica.is_empty() == false {
            self.leader_acceptor_broadcast_chan_sender
                .send(self.messages_from_replica.pop_front().unwrap());
            // go into paused mode only after sending all stipulated messages
            self.slot_in += 1;
        }
    }





    fn processing_messages() {
        
        unimplemented!()

    }





    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Paused => {
                self.operating_state = OperatingState::Paused;
            }

            ControlSignal::Run(num_msgs) => {
                println!("Leader {} activated", self.id);
                self.operating_state = OperatingState::Run(num_msgs);

                // first spawning of the scout
                let (leader_scout_sender, leader_scout_receiver) = unbounded();
                let scout_context = scout::new(
                                    self.scout_id.clone(),
                                    self.id.clone(),
                                    self.scout_acceptor_broadcast_chan_sender.clone(),
                                    leader_scout_receiver,
                                    self.all_scouts_leader_sender.clone(),
                                    self.ballot_num.clone(),
                                );
                scout_context.start(self.num_acceptors.clone());
                self.scout_id  = self.scout_id + 1u16;
                self.leader_to_all_scouts_sender.push(leader_scout_sender);
            }

            ControlSignal::Exit => {
                println!("Exit signal at Leader {} received", self.id);
                self.operating_state = OperatingState::Exit;
            }
        }
    }
}
