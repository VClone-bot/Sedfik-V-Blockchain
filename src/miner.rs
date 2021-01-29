use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use rand::Rng;
use std::str::from_utf8;
use crossbeam_utils::thread;
use std::collections::HashSet;

#[path="./block.rs"]
mod block;

pub struct Miner {
    pub id: u32, // Our ID
    pub network: HashSet<(u32, String)>, // The IDs of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub socket: TcpListener, // Listener for Tcp transactions
    pub sockip: String,
}

pub fn create_miner(socket: String) -> Miner {
    println!("Miner creation...");
    let miner = Miner::new(socket);
    println!("{:?}", &miner);
    return miner;
}

pub fn join_miner(socket: String, destination: String) -> Miner {
    println!("Joining miner...");
    let miner = Miner::new(socket);
    miner.join(destination);
    println!("{:?}", &miner);
    return miner;
}

impl Miner {
   
    ////////// NETWORKING

    /// CONSTRUCTOR
    /// `socket` - an ip:port string representing where is the Miner listening
    /// returns a new Miner with a TcpListener that listens to the given ip:port
    pub fn new (socket: String) -> Self {
        let mut rng = rand::thread_rng();
        return Miner {
            id: rng.gen::<u32>(),
            network: HashSet::new(),
            blocks: Vec::new(),
            socket: TcpListener::bind(&socket).unwrap(),
            sockip: socket.to_string(),
        }
    }

    /// Function to join an existing network
    /// * `destination` - the ip:port of the Miner we want to join
    pub fn join(&self, destination: String) {
        // Connexion au socket distant
        if let Ok(mut stream) = TcpStream::connect(&destination) {
            println!("Réseau {} rejoint !", &destination);

            // Écriture du message à envoyer
            let msg = b"Ping!";
            let resp = b"Pong!";
            stream.write(msg).unwrap();

            println!("Sent ping, awaiting reply...");
            let mut data = [0 as u8; 5]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == resp {
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
            println!("Connexion au réseau {} impossible", &destination);
        }
        println!("Terminé.");
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

    pub fn handle_client(&self, mut stream: TcpStream) {
        let mut data = [0 as u8; 50];
        while match stream.read(&mut data) {
            Ok(_) => {
                println!("Message: {} \nFrom: {}", std::str::from_utf8(&data).unwrap(), stream.peer_addr().unwrap());
                stream.write(b"Pong!").unwrap();
                true
            },
            Err(_) => {
                println!("Une erreur est survenue, fermeture de la connexion");
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
    }

    /// Function add a Miner to the network
    /// `peer_id` - an integer to identify the Miner, should be unique in the network
    /// `peer_addr` - the socket on which the Miner is listening, should be unique aswell
    /// Update the current Miner's network, returns true if the Miner was added to the newtork, false if the Miner was already in the network
    pub fn add_to_network(&mut self, peer_id: u32, peer_addr: String) -> bool {
        self.network.insert((peer_id, peer_addr))
    }

    pub fn listen(&self) {
        println!("Server listening on port {}", &self.sockip);
        // accept connections and process them, spawning a new thread for each one
        for stream in self.socket.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap()); 
                    thread::scope(|s| {
                        s.spawn(move |_| {
                            self.handle_client(stream)
                        });
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            } 
        }
        // close the socket server
        drop(&self.socket);
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
