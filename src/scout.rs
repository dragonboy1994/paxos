use crossbeam::channel::{Sender, Receiver, TryRecvError};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use crate::broadcast_channel::BroadcastSender;
use crate::utils::{Operation, Command, Decision, Ballot, P1a, P1b, P2a, P2b, Adopted, Preempted, ScoutMessage, Pvalue};



pub struct Context {
    // Id of the scout
    scout_id: u16,

    // Id of the leader
    leader_id: u8,

    // broadcast handle to all acceptors
    scout_acceptor_broadcast_chan_sender: BroadcastSender<P1a>,

    // receiving handle of the channels from the leader to the scout
    // for receiving P1b
    leader_scout_receiver: Receiver<P1b>,

    // sending handle of the channel from the scout to the leader
    // for sending adopted and preempted
    scout_leader_sender: Sender<ScoutMessage>,

    // ballot num the scout is responsible for
    ballot_num: Ballot,

    // list of all acceptors that have replied back with P1b
    waitfor: Vec<u8>,

    // list of all pvalues received
    pvalues: Vec<Pvalue>, 

}

pub fn new(
    scout_id: u16,
    leader_id: u8,
    scout_acceptor_broadcast_chan_sender: BroadcastSender<P1a>,
    leader_scout_receiver: Receiver<P1b>,
    scout_leader_sender: Sender<ScoutMessage>,
    ballot_num: Ballot,
) -> Context {
    let context = Context {
        scout_id,
        leader_id,
        scout_acceptor_broadcast_chan_sender,
        leader_scout_receiver,
        scout_leader_sender,
        ballot_num,
        waitfor: Vec::new(),
        pvalues: Vec::new(),
    };

    context
}



impl Context {
    pub fn start(mut self, num_acceptors: u8) {
        println!("Scout initiated - part 2");
        // broadcast the P1a message to all acceptors
        self.scout_acceptor_broadcast_chan_sender
            .send(P1a::create(self.leader_id.clone(), self.ballot_num.clone(), self.scout_id.clone()));

        // thread spawning
        thread::Builder::new()
            .spawn(move || {
                loop {
                    match self.leader_scout_receiver.try_recv() {
                        Ok(message) => {
                            // checking the ballot
                            if message.get_ballot() == self.ballot_num.clone() {
                                // updating pvalues
                                self.pvalues.append(&mut message.get_pvalues());
                                // updating waitfor
                                self.waitfor.push(message.get_acceptor_id());
                                // observe that the following inequality is opposite from in the PMMC
                                if self.waitfor.len() as u8 > num_acceptors/2 {
                                    // sending adopted message
                                    self.scout_leader_sender
                                        .send(ScoutMessage::Adopted(Adopted::create(self.ballot_num.clone(), self.pvalues.clone())));
                                    break;
                                }
                            } else {
                                // sending preempted message
                                self.scout_leader_sender
                                    .send(ScoutMessage::Preempted(Preempted::create(message.get_ballot())));
                                break;
                            }

                        }
                        _ => {}

                    }
                }

            })
            .unwrap();


    }
}