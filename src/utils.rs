use std::cmp::Ordering;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Operation {
    Null,
    Add(i32),
    Subtract(i32),
    Multiply(i32),
}





// the structure of command that is sent by clients to the replicas
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Command {
    client_id: u32,
    command_id: u32,
    operation: Operation,
}

impl Command {
    pub fn create(
        client_id: u32,
        command_id: u32,
        operation: Operation 
    ) -> Command {
        Command{
            client_id,
            command_id,
            operation
        }
    }

    pub fn get_command_id(&self) -> u32 {
        self.command_id.clone()
    }

    pub fn get_operation(&self) -> Operation {
        self.operation.clone()
    }
    
    pub fn get_client_id(&self) -> u32 {
        self.client_id.clone()
    }

}





// structure of ballot number as a lexocographically ordered pair
// need to order Partial ordering
#[derive(Eq, Debug, Clone)]
pub struct Ballot {
    count: u32,
    leader_id: u32,
}

impl Ballot {
    // for creating the initial ballot
    pub fn create(leader_id: u32) -> Ballot {
        Ballot{ count: 0u32, leader_id}
    }

    pub fn increment(&self, leader_id: u32) -> Ballot {
        // increment by 1
        Ballot{ count: self.count + 1u32, leader_id }
    } 

}


impl Ord for Ballot {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.count != other.count {
            self.count.cmp(&other.count)
        } else {
            if self.leader_id < other.leader_id {
                Ordering::Less
            } else if self.leader_id == other.leader_id {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }
    }
}

impl PartialOrd for Ballot {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Ballot {
    fn eq(&self, other: &Self) -> bool {
        (self.count == other.count) && (self.leader_id == other.leader_id)
    }
}







// the structure of pvalues included in the messages between leader and acceptors
#[derive(Debug, Clone)]
pub struct Pvalue {
    ballot: Ballot,
    slot: u32,
    command: Command,
}

impl Pvalue {
    pub fn get_ballot_num(&self) -> Ballot {
        self.ballot.clone()
    }

    pub fn get_slot(&self) -> u32 {
        self.slot.clone()
    }

    pub fn get_command(&self) -> Command {
        self.command.clone()
    }

    pub fn create(
        ballot: Ballot,
        slot: u32,
        command: Command,
    ) -> Pvalue {
        Pvalue{ ballot, slot, command }
    }
}






// sent by clients to replicas
#[derive(Debug, Clone)]
pub struct Request {
    command: Command,
}

impl Request {
    pub fn create(command: Command) -> Request {
        Request{ command }
    }

    pub fn get_command(&self) -> Command {
        self.command.clone()
    }

}






// sent by replicas to clients
pub struct Response {
    command_id: u32,
    result: i32,
}

impl Response{
    pub fn create(command_id: u32, result: i32) -> Response {
        Response{ command_id, result }
    }

    pub fn get_command_id(&self) -> u32 {
        self.command_id.clone()
    }

    pub fn get_result(&self) -> i32 {
        self.result.clone()
    }
}






// sent by replicas to the leaders
#[derive(Debug, Clone)]
pub struct Propose {
    slot: u32,
    command: Command,
}

impl Propose{
    pub fn create(slot: u32, command: Command) -> Propose {
        Propose{ slot, command }
    }

    pub fn get_slot(&self) -> u32 {
        self.slot.clone()
    }

    pub fn get_command(&self) -> Command {
        self.command.clone()
    }
}





// sent by the commander in leaders to the replicas
#[derive(Debug, Clone, Hash)]
pub struct Decision {
    slot: u32,
    command: Command,
}

impl Decision {
    pub fn get_details(self) -> (Command, u32) {
        (self.command, self.slot)
    }

    pub fn create(slot: u32, command: Command) -> Decision {
        Decision{ slot, command }
    }
}






// sent by scout to its leader
#[derive(Debug, Clone)]
pub struct Adopted {
    ballot: Ballot,
    pvalues: Vec<Pvalue>,
}

impl Adopted {
    pub fn create(ballot: Ballot, pvalues: Vec<Pvalue>) -> Adopted {
        Adopted{ ballot, pvalues }
    }

    pub fn get_pvalues(&self) -> Vec<Pvalue> {
        self.pvalues.clone()
    }


    pub fn get_ballot(&self) -> Ballot {
        self.ballot.clone()
    }

}






// sent by scout/commander to its leader
pub struct Preempted {
    ballot: Ballot,
}

impl Preempted {
    pub fn create(ballot: Ballot) -> Preempted {
        Preempted{ ballot }
    }


    pub fn get_ballot(&self) -> Ballot {
        self.ballot.clone()
    }
}




// the message sent by scout thread to the leader
pub enum ScoutMessage {
    Adopted(Adopted),
    Preempted(Preempted),
}






// sent by scout to the acceptor
#[derive(Debug, Clone)]
pub struct P1a {
    leader_id: u32,
    ballot: Ballot,
    scout_id: u32,
}

impl P1a {
    pub fn get_ballot_num(&self) -> Ballot {
        self.ballot.clone()
    }

    pub fn get_leader_id(&self) -> u32 {
        self.leader_id.clone()
    }

    pub fn get_scout_id(&self) -> u32 {
        self.scout_id.clone()
    }

    pub fn create(
        leader_id: u32,
        ballot: Ballot,
        scout_id: u32,       
    ) -> P1a {
        P1a{ leader_id, ballot, scout_id }
    }

}





// sent by acceptor to the scout
#[derive(Debug, Clone)]
pub struct P1b {
    acceptor_id: u32,
    ballot: Ballot,
    // set of pvalues accepted by the acceptor
    accepted: Vec<Pvalue>,
    scout_id: u32,
}

impl P1b {
    pub fn create(
        acceptor_id: u32, 
        ballot: Ballot, 
        accepted: Vec<Pvalue>, 
        scout_id: u32,
    ) ->P1b {
        P1b{ acceptor_id, ballot, accepted, scout_id }
    }

    pub fn get_ballot(&self) -> Ballot {
        self.ballot.clone()
    }

    pub fn get_scout_id(&self) -> usize {
        self.scout_id.clone() as usize
    }
    
    pub fn get_pvalues(&self) -> Vec<Pvalue> {
        self.accepted.clone()
    }

    pub fn get_acceptor_id(&self) -> u32 {
        self.acceptor_id.clone()
    }
}





// sent by commander to the acceptor
#[derive(Debug, Clone)]
pub struct P2a {
    leader_id: u32,
    pvalue: Pvalue,
    commander_id: u32,
}


impl P2a {
    pub fn get_pvalue(&self) -> Pvalue {
        self.pvalue.clone()
    }

    pub fn get_ballot_num(&self) -> Ballot {
        self.pvalue.get_ballot_num()
    }

    pub fn get_leader_id(&self) -> u32 {
        self.leader_id.clone()
    }

    pub fn get_commander_id(&self) -> u32 {
        self.commander_id.clone()
    }

    pub fn create(
        leader_id: u32,
        pvalue: Pvalue,
        commander_id: u32,
    ) -> P2a {
        P2a{ leader_id, pvalue, commander_id }
    }
}







// sent by the acceptor to the commander
#[derive(Debug, Clone)]
pub struct P2b {
    acceptor_id: u32,
    ballot: Ballot,
    commander_id: u32,
}

impl P2b {
    pub fn create(
        acceptor_id: u32, 
        ballot: Ballot,
        commander_id: u32,
    ) -> P2b {
        P2b{ acceptor_id, ballot, commander_id }
    }

    pub fn get_ballot(&self) -> Ballot {
        self.ballot.clone()
    }

    pub fn get_acceptor_id(&self) -> u32 {
        self.acceptor_id.clone()
    }

    pub fn get_commander_id(&self) -> usize {
        self.commander_id.clone() as usize
    }

}
























