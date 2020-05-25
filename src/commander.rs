use crossbeam::channel::{Sender, Receiver, TryRecvError};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use crate::broadcast_channel::BroadcastSender;
use crate::utils::{Operation, Command, Decision, Ballot, P1a, P1b, P2a, P2b, Adopted, Preempted, ScoutMessage, Pvalue};



pub struct Context {

    // Id of the commander
    commander_id: u16,

    // Id of the leader
    leader_id: u8,

    // sending handle of the broadcast channel from commander to replica
    leader_replica_broadcast_chan_sender: BroadcastSender<Decision>,

    // broadcast handle to all acceptors
    commander_acceptor_broadcast_chan_sender: BroadcastSender<P2a>,

    // receiving handle of the channels from the leader to the commander
    // for receiving P2b
    leader_commander_receiver: Receiver<P2b>,

    // sending handle of the channel from the  commander to the leader
    // for sending preempted
    commander_leader_sender: Sender<Preempted>,

    // list of all acceptors that have replied back with P1b
    waitfor: Vec<u8>,

    // ballot num the scout is responsible for
    ballot_num: Ballot,
    
    // slot 
    slot: u8,
    
    // command
    command: Command

}


pub fn new(
    commander_id: u16,
    leader_id: u8,
    leader_replica_broadcast_chan_sender: BroadcastSender<Decision>,
    commander_acceptor_broadcast_chan_sender: BroadcastSender<P2a>,
    leader_commander_receiver: Receiver<P2b>,
    commander_leader_sender: Sender<Preempted>,
    ballot_num: Ballot,
    slot: u8,
    command: Command,
) -> Context {
    let context = Context {
        commander_id,
        leader_id,
        leader_replica_broadcast_chan_sender,
        commander_acceptor_broadcast_chan_sender,
        leader_commander_receiver,
        commander_leader_sender,
        waitfor: Vec::new(),
        ballot_num,
        slot,
        command,
    };

    context
}



impl Context{

    pub fn start(mut self, num_acceptors: u8) {

        // broadcast th P2a message to all acceptors
        self.commander_acceptor_broadcast_chan_sender
            .send(
                P2a::create(
                    self.leader_id,
                    Pvalue::create(self.ballot_num.clone(), self.slot.clone(), self.command.clone()),
                    self.commander_id.clone(),
                )
            );
        // println!("Commander of the leader {} has broadcast P2a message", self.leader_id);

        
        // thread spawning
        thread::Builder::new()
            .spawn( move || {
                loop{
                    match self.leader_commander_receiver.try_recv() {
                        Ok(message) => {
                            // checking the ballot
                            if message.get_ballot() == self.ballot_num.clone() {
                                // updating waitfor
                                self.waitfor.push(message.get_acceptor_id());
                                // observe that the following inequality is opposite from in the PMMC
                                if self.waitfor.len() as u8 > num_acceptors/2 {
                                    // broadcast to all replicas
                                    self.leader_replica_broadcast_chan_sender
                                        .send(Decision::create(self.slot, self.command));
                                    break;
                                } 
                            } else {
                                // sending preempted message
                                self.commander_leader_sender
                                    .send(Preempted::create(message.get_ballot()));
                                break;
                            }
                        }
                        _ => {}
                    }

                }
            }).unwrap();

    }

}