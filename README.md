# Paxos

Paxos is a Rust implementation for multi-Paxos protocol without crash failures. The original Paxos was proposed by Leslie Lamport in his landmark paper `Part-time parliament`. The Paxos variant that this repository implements is from the paper called Paxos made moderately complex (PMMC) .

Note that the code in the repo can't handle crash failures yet -> in the context of PMMC, there is no functionality for handling config commands. I may add it later if this repo gets enough attention.

## Overview
In order to run the 


## Approach I took
Generally, repos don't describe how codebase has been written. In a break from that tradition, this section will describe the approach I took in building this repository. Hopefully, this well serve as a good map for someone else who also wants to build Paxos consensus from scratch.
- Add multiple leaders and one receiver and broadcast channel from receiver to leaders - this channel is just communicating simple u8 messages
- Add multiple replica
- Add client, add broadcast channels from client to replicas, modify replicas so that the replicas broadcast the messages received from the client - these channels are just communicating simple u8 messages
- Add acceptors and communication channels from leaders to acceptors -  these channels are just communicating simple u8 messages
- Write message structs, ordering of ballots
- Add reverse channels - add mpsc channel from replica to client, added broadcast channel from leader to replica
- Add reverse channels - add mpsc channel from acceptors to  leaders
- Update the commands sent by clients
- Change the broadcast channel from client to the replicas - messages will be Request
- Update the replica context
- Change the broadcast channel from leader to the replicas - message



## Some References
1. The original Paxos paper by Lamport  in [Part-time parliament](https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf)
2. The paper this implementation follows is [Paxos made moderately complex](https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf)
