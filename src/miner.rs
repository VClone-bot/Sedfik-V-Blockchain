use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use rand::Rng;
use std::str::from_utf8;
use std::str::FromStr;
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

/// Pimped serialized hashset 
/// 
/// *`set` The hashSet to serialized 
pub fn hashset_to_string(set: &HashSet<(u32, String)>) -> String {
    let mut res = vec![];
    for (id, addr) in set {
        res.push(id.to_string() +","+ &addr.to_string());
        println!("{}, {}",id,addr);
    }
    res.join(";")
}

/// Pimped deserialized hashset
/// 
/// 
pub fn hashset_from_string(hashset :String) -> HashSet<(u32, String)> {
    let mut res = HashSet::<(u32,String)>::new();
    let splited: Vec<&str> = hashset.split(";").collect();
    for element in splited {
        let couple: Vec<&str> = element.split(",").collect();
        let id: u32 =  String::from(couple[0].trim_matches(char::from(0))).parse::<u32>().unwrap();
        let address: String = couple[1].to_string();
        println!("id:{}, addr: {}",&id,&address);
        res.insert((id,address));
    }
    res
}

/// Util
/// Conctene u8 array
/// * `first`
/// * `second`
pub fn concat_u8(first: &[u8], second: &[u8]) -> Vec<u8> {
    [first, second].concat()
}

pub struct Miner {
    pub id: u32, // Our ID
    pub network: HashSet<(u32, String)>, // The IDs of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub sockip: String,
}

pub fn create_miner(socket: String, destination: String) {
    println!("Miner creation...");
    let mut miner = Miner::new(socket.to_string());
    miner.add_to_network(miner.get_id(),socket.to_string());
    println!("{:?}", &miner);
    for (i,e) in &miner.network {
        println!("{}, {}",i,e);
    }
    if !!! destination.is_empty() {
        miner.join(destination);
    }
    miner.listen();
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
        self.send_message(&destination, &"".to_string(),Flag::Connect);
        println!("Join done.");
    }



    /// Function to send a message
    /// * `stream` - Tcp Stream.
    /// * `message` - The message to send.
    pub fn send_message(&self, destination: &String, message: &String, flag: Flag) {
        println!("Sending message: {} \nTo: {}",&message, &destination);
        if let Ok(mut stream) = TcpStream::connect(&destination) {
            let m: &[u8] = &concat_u8(&[flag as u8], &message.as_bytes()[0..]); // TODO : Does the flag still is first byte ?
            stream.write(m);
            println!("Message sended");
        } else {
            println!("Connection with {} failed.", &destination);
        }
    }



    /// Message propagation to all neighbors
    /// * `message` - Message sent.
    pub fn broadcast(&self, message: &String) {
        // For each neighbor
        println!("Propaging: {}", &message);
        for (_, neighbor_address) in &self.network {
            // Open connection with another thread
            thread::scope(|s| {
                s.spawn(move |_| {
                    // Connect to neighbor             
                    self.send_message(&neighbor_address, &message, Flag::Ok); // TODO : Change Flag
                });
            });
        }
    }

    pub fn init_network(&self) {
        
    }

    pub fn handle_client(&mut self, mut stream: TcpStream) {
        let mut data = [0 as u8; 50];
        while match stream.read(&mut data) { 
            Ok(size) if size > 0 => { // If a message is received
                println!("Message received of size: {}", &size);
                let flag = Flag::from_u8(data[0]); // get the flag
                println!("\tFlag: {}", &data[0]);
                let message = std::str::from_utf8(&data[0..size]).unwrap();
                println!("\tMessage: {}", &message);

                let text = &message[1..]; // get the remainder of the message

                // select appropriate response based on the flag, convert the u8 number to flag
                match flag {
                    Flag::Connect => {
                        println!("OK!");
                        let destination = format!("{}:{}",&stream.local_addr().unwrap().ip().to_string(),&stream.local_addr().unwrap().port().to_string());
                        
                        self.send_message(&destination , &hashset_to_string(&self.network), Flag::Ok);
                    }
                    Flag::Disconnect => {
                        let peer_id = text[1..4].parse::<u32>().unwrap();
                        let peer_addr = text[4..].trim().to_string();
                        self.remove_from_network(peer_id, peer_addr);
                    }
                    Flag::Ok => {
                        let received_network: String = std::str::from_utf8(&data[0..]).unwrap().to_string().replace(" ", "");
                        println!("Reply is ok!\nNetwork:{} \n {}", &received_network, &received_network.chars().count());
                        println!("{} ",&received_network.chars().nth(40).unwrap_or('0'));
                        let network: HashSet<(u32,String)> = hashset_from_string(received_network.to_string());
                        for (i,e) in &network {
                            println!("{}, {}",i,e);
                        }
                        // self.network | network;
                        self.network = self.network.union(&network).into_iter().cloned().collect::<HashSet<_>>();
                        println!("New network: ");
                        for (i,e) in &self.network {
                            println!("{}, {}",i,e);
                        }
                    }
                    _ => { println!("Error: flag not recognized"); }
                } 
                data = [0 as u8; 50];
                true
            },
            Ok(size) => { println!("No message received"); false },
            Err(_) => {
                println!("Error occurs, closing connection");
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } 
        {}       
    }

    /// Function to add a Miner to the network
    /// `peer_id` - an integer to identify the Miner, should be unique in the network
    /// `peer_addr` - the socket on which the Miner is listening, should be unique aswell
    /// Update the current Miner's network, returns true if the Miner was added to the newtork, false if the Miner was already in the network
    pub fn add_to_network(&mut self, peer_id: u32, peer_addr: String) -> bool {
        self.network.insert((peer_id, peer_addr))
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
                    println!("New connection: {}", &stream.peer_addr().unwrap());  
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
            //&self.network,
        )
        
    }
}
