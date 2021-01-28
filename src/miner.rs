use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use rand::Rng;

#[path="./block.rs"]
mod block;

pub struct Miner {
    pub id: u32, // Our ID
    pub network: Vec<String>, // The IDs of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub socket: TcpListener, // Listener for Tcp transactions
}

pub fn create_miner(socket: String) {
    println!("Miner creation...");
    let miner = Miner::new(socket);
    println!("{:?}", &miner);
}

pub fn join_miner(socket: String, destination: String) {
    println!("Joining miner...");
    let miner = Miner::new(socket);
    miner.join(destination);
    println!("{:?}", &miner);
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
            socket: TcpListener::bind(socket).unwrap(),
        }
    }

    pub fn join(&self, destination: String) {
        if let Ok(stream) = TcpStream::connect(&destination) {
            println!("Réseau {} rejoint !", &destination);

        } else {
            println!("Connexion au réseau {} impossible", &destination);
        }
    }

    pub fn init_network(&self) {

    }

    pub fn handle_connection(&self, mut stream: TcpStream) {
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
   
}

impl Debug for Miner {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Miner[{}]: \n Network:",
            &self.id,
            // &self.network,
        )
    }
}
