
Paxos is a Rust implementation for multi-Paxos protocol without crash failures. The original Paxos was proposed by Leslie Lamport in his landmark paper https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf. The Paxos variant that this repository implements is from the paper "Paxos made moderately complex (PMMC)" https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf.

Note that the code in the repo can't handle crash failures yet -> in the context of PMMC, there is no functionality for handling config commands. I may add it later if this repo gets enough attention.

## Overview
In order to run the 

## Some References
1. The original Paxos paper by Lamport  in [this]( https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf)
2. 
