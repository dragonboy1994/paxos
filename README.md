
Paxos is a Rust implementation for multi-Paxos protocol without crash failures. The original Paxos was proposed by Leslie Lamport in his landmark paper `Part-time parliament`. The Paxos variant that this repository implements is from the paper called Paxos made moderately complex (PMMC) .

Note that the code in the repo can't handle crash failures yet -> in the context of PMMC, there is no functionality for handling config commands. I may add it later if this repo gets enough attention.

## Overview
In order to run the 


## Approach I took
Generally, repos don't describe how codebase has been written. In a break from that tradition, this section will describe the approach I took in building this repository. Hopefully, this well serve as a good map for someone else who also wants to build Paxos consensus from scratch.
- Add multiple leaders and one receiver and broadcast channel from receiver to leaders

## Some References
1. The original Paxos paper by Lamport  in [Part-time parliament](https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf)
2. The paper this implementation follows is [Paxos made moderately complex](https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf)
