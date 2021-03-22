use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use crossbeam_utils::thread;
use std::collections::HashSet;

#[path="./block.rs"]
mod block;

#[path="./wallet.rs"]
mod wallet;

#[derive(Copy, Clone)]
pub enum Flag {
    /// Ok -> Network
    Ok,
    Connect, // flag to signal that a Miner joined the newtwork
    Disconnect, // flag to signal that a Miner disconnected from the network
    RequireID,
    GiveID,
    BroadcastConnect,
    BroadcastDisconnect,
    Transaction,
    RequireWalletID,
    Check,
}

impl Flag {
    fn from_u8(value: u8) -> Flag {
        match value {
            0 => Flag::Ok,
            1 => Flag::Connect,
            2 => Flag::Disconnect,
            3 => Flag::RequireID,
            4 => Flag::GiveID,
            5 => Flag::BroadcastConnect,
            6 => Flag::BroadcastDisconnect,
            10 => Flag::Transaction,
            11 => Flag::RequireWalletID,
            12 => Flag::Check,
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
    let splitted: Vec<&str> = hashset.split(";").collect();
    for element in splitted {
        let couple: Vec<&str> = element.split(",").collect();
        let id: u32 =  String::from(couple[0].to_string().trim_matches(char::from(0))).parse::<u32>().unwrap();
        let address: String = String::from(couple[1].to_string().trim_matches(char::from(0)));
        res.insert((id,address));
    }
    return res.to_owned();
}

const TRAM_SIZE: usize = 100;

/// Util
/// Conctene u8 array
/// * `first`
/// * `second`
/// Example 
/// ```rust 
/// let first: &[u8] = String::from("Hello, ").as_bytes();
/// let second: &[u8] = String::from("World!").as_bytes();
/// asserteq!("Hello, World!",concat_u8(&first,&second));
/// ```
pub fn concat_u8(first: &[u8], second: &[u8]) -> Vec<u8> {
    [first, second].concat()
}


// Ajoute du padding sockip
pub fn encode_sockip(sockip: String) -> String {
    return format!("{:X<21}", sockip);
}

// Retire le padding au sockip
pub fn decode_sockip(sockip: String) -> String {
    return str::replace(&sockip, "X", "");
}

pub fn encode_id(id: String) -> String {
    return format!("{:Y<10}", id);
}

pub fn decode_id(id: String) -> String {
    return str::replace(&id, "Y", "");
}


pub fn decode_message(msg : &[u8]) -> (Flag, String, String, String){
    println!("SSSDL111 .{:?}.", msg);
    let flag = Flag::from_u8(msg[0]); // get the flag
    let sockip_encoded = std::str::from_utf8(&msg[1..21]).unwrap();
    let id_encoded = std::str::from_utf8(&msg[22..31]).unwrap();
    let msg = std::str::from_utf8(&msg[32..]).unwrap();
    let sockip = decode_sockip(sockip_encoded.to_string());
    (flag, decode_sockip(sockip.to_string()), decode_id(id_encoded.to_string()), msg.to_string())
}

pub fn encode_message(flag : Flag, sockip : String, id : String, msg : String) -> Vec<u8>{
    println!("\nEncoding message");
    let flag_convert: &[u8] = &[flag as u8];
    let sockip_convert : String = encode_sockip(sockip);
    let id_convert : String = encode_id(id);
    println!("\tmessage to encode: {}",&msg);
    let msg_convert : &[u8] = msg.as_bytes();
    println!("\tmessage encoded: {:?}",&msg_convert);
    concat_u8(flag_convert, &concat_u8(sockip_convert.as_bytes(), &concat_u8(id_convert.as_bytes(), msg_convert)))
}

pub fn create_miner(miner_type: char, socket: String, destination: String) {
    println!("Miner creation...");
    let mut miner;
    match miner_type {
        'c' => { miner = Miner::new(0, socket.to_string()); }
        'j' => { miner = Miner::new(ask_for_id(&socket, &destination), socket.to_string()); }
        _ => { println!("Unrecognized miner type"); return (); }
    }
    miner.add_to_network(miner.get_id(),socket.to_string());
    println!("{:?}", &miner);
    for (i,e) in &miner.network {
        println!("{}, {}",i,e);
    }
    if !!! destination.is_empty() {
        println!("Now connecting to network...");
        miner.join(destination);
        println!("Connected!\n");
    }
    println!("Starting to listen...");
    miner.listen();
}

pub fn ask_for_id(socket: &String, destination: &String) -> u32 {
    println!("Asking {} for id...", &destination);
    let listener = TcpListener::bind(&socket).unwrap();
    let mut id: u32 = 0;

    if let Ok(mut stream) = TcpStream::connect(&destination) {
        let m: &[u8] = &encode_message(Flag::RequireID, socket.to_string(), "".to_string(), "".to_string());
        match stream.write(m) {
            Ok(_) => { println!("Asked for id"); }
            Err(e) => { println!("Error: {}", e); }
        }
        println!("Message sended");
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Getting ID from Genesis");
                id = handle_id(stream);
                println!("My ID is {}.", &id);
                return id;
            }
            Err(e) => {
                println!("Error: {}", e);
                return 0;
            }
        }
    }
    return id;
} 

pub fn handle_id(mut stream: TcpStream) -> u32 {
    let mut data = [0 as u8; 50];
    match stream.read(&mut data) {
        Ok(size) if size > 0 => {
            let tuple : (Flag, String, String, String) = decode_message(&data);
            let id_as_str_decoded = decode_id(std::str::from_utf8(tuple.2.as_bytes()).unwrap().to_owned());
            print!("tuple:{},{},{},{}",&data[0],tuple.1,tuple.2,tuple.3);
            let id = 3;//id_as_str_decoded.parse::<u32>().unwrap();
            return id;
        },
        Ok(_) => { println!("No message received");},
        Err(e) => {
            println!("Error occured, closing connection: {}", e);
            stream.shutdown(Shutdown::Both).unwrap();
        }
    }
    {}
    0
}

pub struct Miner {
    pub id: u32, // Our ID
    pub network: HashSet<(u32, String)>, // The IDs and adresses of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub sockip: String,
    pub wallets: HashSet<(u32, String)>,
}

impl Miner {

    /// CONSTRUCTOR
    /// `socket` - an ip:port string representing where is the Miner listening
    /// returns a new Miner with a TcpListener that listens to the given ip:port
    pub fn new (id: u32, socket: String) -> Self {
        return Miner {
            id: id,
            network: HashSet::new(),
            blocks: Vec::new(),
            sockip: socket.to_string(),
            wallets: HashSet::new(),
        }        
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }


    pub fn display_network(&self) {
        println!("Current network:");
        for miner in &self.network {
            println!("\tid: {}, sockip: {}", miner.0, miner.1);
        }
    }

    /// Function to join an existing network
    /// * `destination` - the ip:port of the Miner we want to join
    pub fn join(&self, destination: String) {
        // Connexion au socket distant
        match self.send_message(&destination, &self.id.to_string(), Flag::Connect) {
            Ok(_) => println!("Join done."),
            Err(e) => println!("Err: {}", e),
        }
        
    }

    
    /// Function to send a message
    /// * `stream` - Tcp Stream.
    /// * `message` - The message to send.
    pub fn send_message(&self, destination: &String, message: &String, flag: Flag) -> Result<u8,&'static str> {
        let f = flag as u8;
        println!("Sending message: {} \nTo: {} .. {} \nWith Flag: {}",&message, &destination, &destination.chars().count(),&f);
        match TcpStream::connect(&destination) {
            Ok(mut stream) => {
                println!("Connection established.");
                let m: &[u8] = &encode_message(flag, self.sockip.to_string(),self.id.to_string(), message.to_string());
                println!("Byte message: {:?}",&m);
                match stream.write(m) {
                    Ok(_) => println!("Message writen in buffer"),
                    Err(e) => println!("Error during writing: {}",e.to_string()),
                }
                println!("Message sended");
                return Ok(0);
            }
            Err(e) => {
                println!("Err: {}, during connection",e);
                return Err("Connection failed.");
            } 
        }
    }

    pub fn broadcast_to_network(&self, message: &String, flag: Flag, sender: String) {
        println!("Broadcasting network changes");
        for(_, peer_addr) in &self.network {
            if peer_addr.to_string() != sender {
                match self.send_message(&peer_addr.to_string(), message, flag) {
                    Ok(_) => println!(""),
                    Err(e) => println!("Err: {}",e),
                }
            }
        }
    }

    /// Message propagation to all neighbors
    /// * `message` - Message sent.
    /// Unused
    pub fn broadcast_threaded(&self, message: &String) {
        // For each neighbor
        println!("Broadcasting the message {}", &message);
        for (_, neighbor_address) in &self.network {
            // Open connection with another thread
            thread::scope(|s| {
                s.spawn(move |_| {
                    // Connect to neighbor             
                    match self.send_message(&neighbor_address, &message, Flag::Ok) {
                        Ok(_) => println!(""),
                        Err(e) => println!("Err: {}", e),
                    }
                });
            });
        }
    }

    pub fn retrieve_next_id(&self) -> u32 {
        let mut max_id = &self.id;
        for (id, _) in &self.network {
            if id > max_id {
                max_id = id;
            }
        }
        println!("found id is {}", max_id);
        return (max_id+1).to_owned();
    }

    pub fn retrieve_next_wallet_id(&self) -> u32 {
        let mut max_id = 0;
        for (id, _) in &self.wallets {
            if id > &max_id {
                max_id = *id;
            }
        }
        println!("found id is {}", max_id);
        return (max_id+1).to_owned();
    }

    pub fn handle_client(&mut self, mut stream: TcpStream) {
        let mut data = [0 as u8; TRAM_SIZE];
        while match stream.read(&mut data) { 
            Ok(size) if size > 0 => { // If a message is received
                println!("Message received of size: {}", &size);
                let tuple : (Flag, String, String, String) = decode_message(&data);
                //let flag = Flag::from_u8(data[0]); // get the flag
                let flag = tuple.0;
                println!("\tFlag: {}", &data[0]);
                //println!("\tFlag: {:?}", &flag);
                //let message = std::str::from_utf8(&data[0..size]).unwrap();
                let message = tuple.3;
                println!("\tMessage: {}", &message);

                let text = &message[1..]; // get the remainder of the message
                let sender_sockip = tuple.1;
                println!("\tSockIp: {}", &sender_sockip);

                let sender_id_as_str = tuple.2;
                

                // select appropriate response based on the flag, convert the u8 number to flag
                match flag {
                    Flag::Connect => {
                        println!("Connect Flag received");
                        //let destination = format!("{}:{}",&stream.local_addr().unwrap().ip().to_string(),&stream.local_addr().unwrap().port().to_string());
                        let destination = &sender_sockip;
                        match self.send_message(&destination , &hashset_to_string(&self.network), Flag::Ok) {
                            Ok(_) => println!(""),
                            Err(e) => println!("Err: {}", e),
                        }
                        println!("Sender id: {}", sender_id_as_str);
                        let sender_id = sender_id_as_str.parse::<u32>().unwrap();
                        let broadcast_message = format!("{};{}", sender_sockip, sender_id);
                        println!("Broadcasting message: {}", broadcast_message);
                        self.broadcast_to_network(&broadcast_message, Flag::BroadcastConnect, self.sockip.to_string());
                        self.add_to_network(sender_id, sender_sockip);
                    }
                    Flag::Disconnect => {
                        println!("Disconnect Flag received");
                        let sender_id = sender_id_as_str.parse::<u32>().unwrap();
                        self.remove_from_network(sender_id, sender_sockip.to_owned());
                        let broadcast_message = format!("{};{}", sender_id, sender_id);
                        self.broadcast_to_network(&broadcast_message, Flag::BroadcastDisconnect, self.sockip.to_string());
                    }
                    Flag::Ok => {
                        println!("Ok Flag received");
                        let received_network: &String = &message;
                        println!("Reply is ok!\nNetwork: {} \n Count: {}", received_network, received_network.chars().count());
                        let network: HashSet<(u32,String)> = hashset_from_string(received_network.to_string());

                        self.network = self.network.union(&network).into_iter().cloned().collect::<HashSet<_>>();
                        println!("New network: ");
                        for (i,e) in &self.network {
                            println!("{}, {}",i,e);
                        }
                        // self.broadcast(&message, flag);
                    }
                    Flag::RequireID => {
                        println!("RequireID Flag received");
                        let next_id = self.retrieve_next_id().to_string();
                        match self.send_message(&sender_sockip, &next_id, Flag::GiveID) {
                            Ok(_) => println!("ID correctement envoyé"),
                            Err(e) => println!("Err: {}", e),
                        }
                    }
                    Flag::BroadcastConnect => {
                        println!("BroadcastConnect Flag received");
                        let splitted: Vec<&str> = message.split(";").collect();
                        let new_sockip = String::from(splitted[0].to_string().trim_matches(char::from(0)));
                        let new_id_as_str = String::from(splitted[1].to_string().trim_matches(char::from(0)));
                        println!("id:{}, sockip:{}", new_id_as_str.to_string(), new_sockip.to_string());
                        println!("The message is: -{}-", &message);
                        let new_id = new_id_as_str.parse::<u32>().unwrap();
                        
                        if self.add_to_network(new_id, new_sockip.to_string()) {
                            self.broadcast_to_network(&message, Flag::BroadcastConnect, sender_sockip);
                        }
                    }
                    Flag::RequireWalletID => {
                        println!("Required Wallet ID Flag Received");
                        let next_id = self.retrieve_next_wallet_id();
                        match self.send_message(&sender_sockip, &next_id.to_string(), Flag::GiveID) {
                            Ok(_) => println!("ID Wallet bien envoyé"),
                            Err(e) => println!("Err: {}", e),
                        }
                        self.add_to_wallets(next_id, sender_sockip);
                    }
                    _ => { println!("Error: flag not recognized"); }
                } 
                data = [0 as u8; TRAM_SIZE];
                true
            },
            Ok(_) => { println!("No message received"); false },
            Err(e) => {
                println!("Error occurs, closing connection: {}", e);
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
            self.display_network();
        }
        // close the socket server
        println!("Closing listener");
        drop(listener);
    }

    pub fn ask_for_id(socket: &String, destination: &String) -> u32 {
        println!("Asking {} for id...", &destination);
        let listener = TcpListener::bind(&socket).unwrap();
        let mut id: u32 = 0;
    
        if let Ok(mut stream) = TcpStream::connect(&destination) {
            let m: &[u8] = &encode_message(Flag::RequireID, socket.to_string(), "".to_string(), "".to_string());
            match stream.write(m) {
                Ok(_) => { println!("Asked for id"); }
                Err(e) => { println!("Error: {}", e); }
            }
            println!("Message sended");
        }
    
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Getting Miner ID from Genesis");
                    id = handle_id(stream);
                    println!("My ID is {}.", &id);
                    return id;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    return 0;
                }
            }
        }
        return id;
    } 

    /// Function to give a connecting wallet its id
    pub fn ask_miner_for_wallet_id(socket: &String, miner: &String) -> u32 {
        println!("Asking {} for wallet ID", &miner);
        let listener = TcpListener::bind(&socket).unwrap();
        let mut id: u32 = 0;

        if let Ok(mut stream) = TcpStream::connect(&miner) {
            let m: &[u8] = &encode_message(Flag::RequireWalletID, socket.to_string(), "".to_string(), "".to_string());
            match stream.write(m) {
                Ok(_) => { println!("Asked for id"); }
                Err(e) => { println!("Error: {}", e); }
            }
            println!("Message sended, wallet added to miner's wallet list");
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Getting Wallet ID from Miner");
                    id = handle_id(stream);
                    println!("My ID is {}", &id);
                    return id;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    return 0;
                }
            }
        }
        return id;
    }

    pub fn add_to_wallets(&mut self, peer_id: u32, peer_addr: String) -> bool {
        self.wallets.insert((peer_id, peer_addr))
    }    
}

impl Debug for Miner {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Miner[{}]: \n Network:",
            &self.id,
            //self.display_network(),
        )
        
    }
}
