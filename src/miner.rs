use std::net::{TcpStream, TcpListener};

#[path="./block.rs"]
mod block;

pub struct Miner {
    pub id: u32, // Our ID
    pub network: Vec<u32>, // The IDs of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub socket: TcpListener, // Listener for Tcp transactions
    pub connection: Option<TcpStream>, // ID of the miner to which we are connected
}

impl Miner {
   
    ////////// NETWORKING

    /** 
     * Dispatch between joining and creating a new network
     */
    pub fn new (&self, role: String, socket: String, connection: String) -> Self {
        if role == "creator" {
            return self.create_network(socket, connection);
        } else if role == "joiner" {
            return self.join_network(socket, connection);
        }
    }

    /** 
     * Used to create a new network if the miner is the genesis of the network
    */
    pub fn create_network(&self, socket: String, connection: String) -> Self {
        return Miner {
            0,
            Vec::new(),
            Vec::new(),
            TcpListener::bind(socket).unwrap();,
            None,
        }
    }

    /** 
     * Used to join an existing network, ask for a copy everyone's network to the network
    */
    pub fn join_network(socket: String, connection: String) -> Self {
        return Miner {
            0,
            Vec::new(),
            Vec::new();
            TcpListener::bind(socket).unwrap();,
            TcpStream::connect("127.0.0.1:34254")?,
        }
    }

    pub fn init_network(self) {

    }

    pub fn handle_connection(self, stream: TcpStream) {
         
    }
   
}
