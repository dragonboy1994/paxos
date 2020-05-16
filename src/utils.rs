use std::cmp::Ordering;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Operation {
    Null,
    Add(u32),
    Subtract(u32),
    Multiply(u32),
}

// the structure of command that is sent by clients to the replicas
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Command {
    client_id: u8,
    command_id: u8,
    operation: Operation,
}

// structure of ballot number as a lexocographically ordered pair
// need to order Partial ordering
#[derive(Eq, Debug, Clone)]
pub struct Ballot {
    count: u16,
    leader_id: u8,
}

// the structure of pvalues included in the messages between leader and acceptors
#[derive(Debug, Clone)]
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
    result: u32,
}

// sent by replicas to the leaders
pub struct Propose {
    slot: u8,
    command: Command,
}

// sent by the commander in leaders to the replicas
#[derive(Debug, Clone, Hash)]
pub struct Decision {
    slot: u8,
    command: Command,
}

// sent by scout to its leader
#[derive(Debug, Clone)]
pub struct Adopted {
    ballot: Ballot,
    pvalues: Vec<Pvalue>,
}

// sent by scout/commander to its leader
pub struct Preempted {
    ballot: Ballot,
}

// sent by scout to the acceptor
#[derive(Debug, Clone)]
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

    pub fn get_operation(&self) -> Operation {
        self.operation.clone()
    }
    
    pub fn get_client_id(&self) -> u8 {
        self.client_id.clone()
    }

}

impl Request {
    pub fn create(command: Command) -> Request {
        Request{ command }
    }

    pub fn get_command(&self) -> Command {
        self.command.clone()
    }

}



impl Response{
    pub fn create(command_id: u8, result: u32) -> Response {
        Response{ command_id, result }
    }
}


impl Propose{
    pub fn create(slot: u8, command: Command) -> Propose {
        Propose{ slot, command }
    }
}


impl Decision {
    pub fn get_details(self) -> (Command, u8) {
        (self.command, self.slot)
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
