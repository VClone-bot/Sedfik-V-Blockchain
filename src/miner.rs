use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::collections::HashSet;
use std::env;

/** Miner
 Gestion des sockets



*/

pub struct Miner {
    pub id: u32,
    pub network: HashSet
}



impl Miner {
   
    fn new (id: u32,network : HashSet) -> Self {

        self.init_network();
        // id = max(network)+1;
        

        return Miner {
           id,
           network,
        }
    }

    fn init_network() {
        // Ping all neigbhors to create first network map 
    }

    fn handle_connection(mut stream: TcpStream) {
         
    }
    
    fn get_network() -> HashSet<u32> {
        &self.network;
    }
   
}
