use crossbeam::channel::{Sender, Receiver, TryRecvError};
use std::thread;

use crate::utils::{P1a, P1b, P2a, P2b, Ballot, Pvalue};

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
    // ID of the leader
    id: u32,


    // ballot number
    ballot_num: Option<Ballot>,

    // accepted set of pvalues
    accepted: Vec<Pvalue>,

    // handle for the broadcast channel between all scouts and the acceptor
    scout_acceptor_broadcast_chan_receiver: Vec<Receiver<P1a>>,

    // handle for the broadcast channel between all commanders and the acceptor
    commander_acceptor_broadcast_chan_receiver: Vec<Receiver<P2a>>,

    // vec of handle for the mpsc channels from the acceptor to all the leaders for the commanders
    // the sender handle is shared with other acceptors
    acceptor_leader_for_commander_mpsc_chan_senders: Vec<Sender<P2b>>,

    // vec of handle for the mpsc channels from the acceptor to all the leaders for the scouts
    // the sender handle is shared with other acceptors
    acceptor_leader_for_scout_mpsc_chan_senders: Vec<Sender<P1b>>,

    // handle for controlling the leader operating state
    control_chan_receiver: Receiver<ControlSignal>,

    // operating state of the leader
    operating_state: OperatingState,
}

pub fn new(
    id: u32,
    scout_acceptor_broadcast_chan_receiver: Vec<Receiver<P1a>>,
    commander_acceptor_broadcast_chan_receiver: Vec<Receiver<P2a>>,
    acceptor_leader_for_commander_mpsc_chan_senders: Vec<Sender<P2b>>,
    acceptor_leader_for_scout_mpsc_chan_senders: Vec<Sender<P1b>>,
    control_chan_receiver: Receiver<ControlSignal>,
) -> Context {
    let context = Context {
        id,
        ballot_num: None,
        accepted: Vec::new(),
        scout_acceptor_broadcast_chan_receiver,
        commander_acceptor_broadcast_chan_receiver,
        acceptor_leader_for_commander_mpsc_chan_senders,
        acceptor_leader_for_scout_mpsc_chan_senders,
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
                            // println!("Acceptor {} in paused mode", self.id);
                            let signal = self.control_chan_receiver.recv().unwrap();
                            self.control_signal_processing(signal);
                        }

                        OperatingState::Run(num_msgs) => {
                            // analyzing under various control channel state
                            match self.control_chan_receiver.try_recv() {
                                Ok(signal) => {
                                    // Exit signal received
                                    self.control_signal_processing(signal);
                                }

                                // empty control channel, feel free to continue interacting with the replicas
                                Err(TryRecvError::Empty) => {
                                    self.processing_p1a_message_from_scout();
                                    self.processing_p2a_message_from_commander();
                                }

                                // Disconnected control channel
                                Err(TryRecvError::Disconnected) => {
                                    panic!("Acceptor {} control channel detached", self.id)
                                }
                            }
                        }

                        OperatingState::Exit => {
                            println!("Acceptor {} deactivated.......................", self.id);
                            break;
                        }
                    }
                }
            })
            .unwrap();
    }






    fn processing_p1a_message_from_scout(&mut self) {
        for handle in &self.scout_acceptor_broadcast_chan_receiver {
            match handle.try_recv() {
                Ok(message) => {
                    // ballot check
                    // println!("Acceptor {} has received P1a", self.id);
                    match self.ballot_num.clone() {
                        Some(b) => {
                            if message.get_ballot_num() > b {
                                self.ballot_num = Some(message.get_ballot_num());
                            }
                        }

                        None => { self.ballot_num = Some(message.get_ballot_num()); }
                    }


                    // send the P1b message to the scout
                    self.acceptor_leader_for_scout_mpsc_chan_senders[message.get_leader_id() as usize]
                    .send( P1b::create(
                                        self.id.clone(),
                                        self.ballot_num.clone().unwrap(), 
                                        self.accepted.clone(), 
                                        message.get_scout_id(),
                                    ));
                    // println!("Acceptor {} has sent P1b", self.id);
                }
                _ => {}
            }
        }
    }


    fn processing_p2a_message_from_commander(&mut self) {
        for handle in &self.commander_acceptor_broadcast_chan_receiver {
            match handle.try_recv() {
                Ok(message) => {
                    // println!("Acceptor {} has received P2a", self.id);
                    if message.get_ballot_num() == self.ballot_num.clone().unwrap() {
                        // inserting the pvalue
                        self.accepted.push(message.get_pvalue());
                    }

                    // send the P2b message to the commander
                    self.acceptor_leader_for_commander_mpsc_chan_senders[message.get_leader_id() as usize]
                    .send(P2b::create(self.id.clone(), self.ballot_num.clone().unwrap(), message.get_commander_id()));
                    // println!("Acceptor {} has sent P2b", self.id);
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
                // println!("Exit signal at Acceptor {} received", self.id);
                self.operating_state = OperatingState::Exit;
            }
        }
    }
}
