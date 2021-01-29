use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use rand::Rng;
use std::str::from_utf8;
use crossbeam_utils::thread;
use std::collections::HashSet;

#[path="./block.rs"]
mod block;

enum Flag {
    /// Ok -> Network
    Ok,
    Connect, // flag to signal that a Miner joined the newtwork
    Disconnect, // flag to signal that a Miner disconnected from the network
}

impl Flag {
    fn from_u8(value: u8) -> Flag {
        match value {
            0 => Flag::Ok,
            1 => Flag::Connect,
            2 => Flag::Disconnect,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

pub struct Miner {
    pub id: u32, // Our ID
    pub network: HashSet<(u32, String)>, // The IDs of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub sockip: String,
}

pub fn create_miner(socket: &String) -> Miner {
    println!("Miner creation...");
    let mut miner = Miner::new(socket.to_string());
    println!("{:?}", &miner);
    
    miner.add_to_network(&miner.get_id(),&socket);

    return miner;
}

pub fn join_miner(socket: String, destination: String) -> Miner {
    println!("Joining miner...");
    
    let miner = create_miner(&socket);

    miner.join(destination);
    println!("{:?}", &miner);
    return miner;
}

impl Miner {

    /// CONSTRUCTOR
    /// `socket` - an ip:port string representing where is the Miner listening
    /// returns a new Miner with a TcpListener that listens to the given ip:port
    pub fn new (socket: String) -> Self {
        let mut rng = rand::thread_rng();
        return Miner {
            id: rng.gen::<u32>(),
            network: HashSet::new(),
            blocks: Vec::new(),
            sockip: socket.to_string(),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// Function to join an existing network
    /// * `destination` - the ip:port of the Miner we want to join
    pub fn join(&self, destination: String) {
        // Connexion au socket distant
        if let Ok(mut stream) = TcpStream::connect(&destination) {
            println!("New miner {} joined", &destination);
            
            // Écriture du message à envoyer
            let connect_flag = Flag::Connect as u8;
            // let message = connect_flag
            // let msg = b"Ping!";
            // let resp = b"Pong!";
            
            stream.write(&connect_flag).unwrap();

            println!("Sent ping, awaiting reply...");
            
            let mut data = [0 as u8; 5]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if data[0] == Flag::Ok as u8 {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        } else {
            println!("Failed to connect with {}", &destination);
        }
        println!("Join done.");
    }

    /// Function to send a message
    /// * `stream` - Tcp Stream.
    /// * `message` - The message to send.
    pub fn send_message(&self, mut stream: TcpStream, message: &String) {
        stream.write(&message.as_bytes()[0..]);
        println!("Message: {} \nTo: {}",&message, stream.peer_addr().unwrap());
    }

    /// Message propagation to all neighbors
    /// * `message` - Message sent.
    pub fn propagate(&self, message: &String) {
        // For each neighbor
        println!("Propaging: {}", message);
        for (_, neighbor_address) in &self.network {
            // Open connection with another thread
            thread::scope(|s| {
                s.spawn(move |_| {
                    // Connect to neighbor
                    let stream = TcpStream::connect(&neighbor_address)
                        .expect("Error : Couldn't connect to miner.");             
                    self.send_message(stream, message);
                });
            });
        }
    }

    pub fn init_network(&self) {
        
    }

    pub fn handle_client(&mut self, mut stream: TcpStream) {
        let mut data = [0 as u8; 50];
        while match stream.read(&mut data) {
            Ok(size) => {
                let message = std::str::from_utf8(&data[0..size]).unwrap();
                let flag = Flag::from_u8(message[0..1].parse::<u8>().unwrap()); // get the flag
                let text = &message[1..]; // get the remainder of the message

                // select appropriate response based on the flag, convert the u8 number to flag
                match flag {
                    Flag::Disconnect => { 
                        let peer_id = text[1..4].parse::<u32>().unwrap();
                        let peer_addr = text[4..].trim().to_string();
                        self.remove_from_network(peer_id, peer_addr);
                    }
                    _ => { println!("Error: flag not recognized"); }
                } 
                true
            },
            Err(_) => {
                println!("Une erreur est survenue, fermeture de la connexion");
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
    }

    /// Function to add a Miner to the network
    /// `peer_id` - an integer to identify the Miner, should be unique in the network
    /// `peer_addr` - the socket on which the Miner is listening, should be unique aswell
    /// Update the current Miner's network, returns true if the Miner was added to the newtork, false if the Miner was already in the network
    pub fn add_to_network(&mut self, peer_id: &u32, peer_addr: &String) -> bool {
        self.network.insert((*peer_id, peer_addr.to_string()))
    }

    /// Function to remove a Miner from the network
    /// `peer_id` - an integer to identify the Miner
    /// `peed_addr` - the socket of the Miner we want to remove from the network
    /// Update the current Miner's network, returns true if the Miner was deleted from the newtork, false if the Miner wasn't in the network
    pub fn remove_from_network(&mut self, peer_id: u32, peer_addr: String) -> bool {
        self.network.remove(&(peer_id, peer_addr))
    }

    /// Function to listen for incoming Streams from the network
    /// Read the stream and spawn a thread to handle the received data
    pub fn listen(mut self) {
        println!("Server listening on port {}", &self.sockip);
        let listener = TcpListener::bind(&self.sockip).unwrap();
        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());  
                    self.handle_client(stream);  
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

}

impl Debug for Miner {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Miner[{}]: \n Network:",
            &self.id,
            // &self.network,
        )
    }
}
