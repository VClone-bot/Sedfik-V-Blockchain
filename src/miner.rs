use std::net::{TcpStream, TcpListener};
use std::thread;

#[path="./block.rs"]
mod block;

pub struct Miner {
    pub id: u32, // Our ID
    pub network: Vec<u32>, // The IDs of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub socket: Option<TcpListener>, // Listener for Tcp transactions
    pub connection: Option<u32>, // ID of the miner to which we are connected
}

impl Miner {
   
    pub fn new () -> Self {
        return Miner {
           id: 0,
           network: Vec::new(),
           blocks: Vec::new(),
           socket: None,
           connection: None,
        }
    }

    ////////// NETWORKING

    /** 
     * Used to create a new network if the miner if the genesis of the network
    */
    pub fn create_network(self) {
        
    }

    /** 
     * Used to join an existing network, ask for a copy everyone's network to the network
    */
    pub fn join_network(self) {

    }

    pub fn init_network(self) {

    }

    pub fn handle_connection(self, stream: TcpStream) {
         
    }
   
}
