use std::net::{TcpStream};
use std::collections::HashSet;
use rand::Rng;
use std::fmt::{ self, Debug, Formatter};
use std::string::String;

/** Miner
 Gestion des sockets
*/

pub struct Miner {
    pub id: u32,
    pub network: HashSet<u32>,
}

pub fn create_miner(address: &String) {
    println!("Miner creation...");
    let miner = Miner::new();
    println!("{:?}", &miner);
    miner.listen(address);

}

impl Miner {
   
    fn new () -> Self {
        let mut rng = rand::thread_rng();
        
        // self.init_network();
        // id = max(network)+1;
        
        return Miner {
           id: rng.gen::<u32>(),
           network: HashSet::<u32>::new(),
        }
    }

   

    fn init_network() {
        // Ping all neigbhors to create first network map 
    }

    fn handle_connection(mut stream: TcpStream) {
         
    }

    fn listen(&self, address: std::String) {
    let listener = TcpListener::bind(address).unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Miner listening on {}", address);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                
                println!("New connection: {}", stream.peer_addr().unwrap());
                handle_connection(stream);
                
                //thread::spawn(move|| {
                    // connection succeeded
                //});
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
    }


    
    pub fn get_network(&self) -> &HashSet<u32> {
        return &self.network;
    }
   
    pub fn get_id(&self) -> &u32 {
        return &self.id;
    } 
}

impl Debug for Miner {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Miner[{}]: \n Network:",
            &self.id,
            // &self.network,
        )
    }
}
