use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub enum Operation {
    Null,
    Add(f64),
    Subtract(f64),
    Multiply(f64),
    Divide(f64),
}

// the structure of command that is sent by clients to the replicas
#[derive(Debug, Clone)]
pub struct Command {
    client_id: u8,
    command_id: u8,
    operation: Operation,
}

// structure of ballot number as a lexocographically ordered pair
// need to order Partial ordering
#[derive(Eq)]
pub struct Ballot {
    count: u16,
    leader_id: u8,
}

// the structure of pvalues included in the messages between leader and acceptors
pub struct Pvalue {
    ballot: Ballot,
    slot: u8,
    command: Command,
}

// sent by clients to replicas
#[derive(Debug, Clone)]
pub struct Request {
    command: Command,
}

// sent by replicas to clients
pub struct Response {
    command_id: u8,
    result: f64,
}

// sent by replicas to the leaders
pub struct Propose {
    slot: u8,
    command: Command,
}

// sent by the commander in leaders to the replicas
pub struct Decision {
    slot: u8,
    command: Command,
}

// sent by scout to its leader
pub struct Adopted {
    ballot: Ballot,
    pvalues: Vec<Pvalue>,
}

// sent by scout/commander to its leader
pub struct Preempted {
    ballot: Ballot,
}

// sent by scout to the acceptor
pub struct P1a {
    leader_id: u8,
    ballot: Ballot,
}

// sent by acceptor to the scout
pub struct P1b {
    acceptor_id: u8,
    ballot: Ballot,

    // set of pvalues accepted by the acceptor
    accepted: Vec<Pvalue>,
}

// sent by commander to the acceptor
pub struct P2a {
    leader_id: u8,
    pvalue: Pvalue,
}

// sent by the acceptor to the commander
pub struct P2b {
    acceptor_id: u8,
    ballot: Ballot,
}


impl Command {
    pub fn create(
        client_id: u8,
        command_id: u8,
        operation: Operation 
    ) -> Command {
        Command{
            client_id,
            command_id,
            operation
        }
    }

    pub fn get_command_id(&self) -> u8 {
        self.command_id.clone()
    }
}

impl Request {
    pub fn create(command: Command) -> Request {
        Request{ command }
    }

    pub fn get_command(self) -> Command {
        self.command
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
