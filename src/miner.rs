use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use rand::Rng;
use std::str::from_utf8;
use crossbeam_utils::thread;

#[path="./block.rs"]
mod block;

pub struct Miner {
    pub id: u32, // Our ID
    pub network: Vec<String>, // The IDs of every member of the network, always unique
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

    // CTOR
    pub fn new (socket: String) -> Self {
        let mut rng = rand::thread_rng();
        return Miner {
            id: rng.gen::<u32>(),
            network: Vec::new(),
            blocks: Vec::new(),
            socket: TcpListener::bind(&socket).unwrap(),
            sockip: socket.to_string(),
        }
    }

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

    /** Send message */
    pub fn send_message(&self, mut stream: TcpStream, message: &String) {
    
        stream.write(&message.as_bytes()[0..]);
        
    }

    /** Propagation */
    pub fn propagate(&self, message: String) {
        // For each neighbor
        for (id, neighbor_address) in self.network {
            
            // Open connection
            
            thread::scope(|s| {
                
                s.spawn(move |_| {
                // connection succeeded
               
                    if let Ok(mut stream) = TcpStream::connect(&neighbor_address) {
                
                        println!("Réseau {} rejoint !", &neighbor_address);
                        self.send_message(stream, &message);
                        
                    } 
                });
            });
        }
    }



    pub fn init_network(&self) {

    }

    pub fn handle_client(&self, mut stream: TcpStream) {
        let mut data = [0 as u8; 50];
        while match stream.read(&mut data) {
            Ok(size) => {
                stream.write(&data[0..size]).unwrap();
                true
            },
            Err(_) => {
                println!("Une erreur est survenue, fermeture de la connexion");
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
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
                        // connection succeeded
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
