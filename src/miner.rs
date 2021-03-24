use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use crossbeam_utils::thread;
// use std::thread;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Sha512, Digest};
//use log::{info, warn};
use std::convert::TryFrom;
use std::sync::{Mutex, Arc};
use std::str::FromStr;

#[path="./block.rs"] mod block;

#[path="./wallet.rs"]
mod wallet;

/// Used for signaling what kind of requests we are sending when networking
/// 
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
    Check,
    Ack,
    Block,
    Transaction,
    MineTransaction,
    OkMineTransaction,
    RequireWalletID,
    RequireBlockchain,
    SendBlockchain,
}


impl Flag {
    /// Simple function to convert a integer to a Flag
    /// Used when receiving/sending a message as primary types are easier to pass than objects
    /// 
    fn from_u8(value: u8) -> Flag {
        match value {
            0 => Flag::Ok,
            1 => Flag::Connect,
            2 => Flag::Disconnect,
            3 => Flag::RequireID,
            4 => Flag::GiveID,
            5 => Flag::BroadcastConnect,
            6 => Flag::BroadcastDisconnect,
            7 => Flag::Check,
            8 => Flag::Ack,
            9 => Flag::Block,
            10 => Flag::Transaction,
            11 => Flag::MineTransaction,
            12 => Flag::OkMineTransaction,
            13 => Flag::RequireWalletID,
            14 => Flag::RequireBlockchain,
            15 => Flag::SendBlockchain,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

/// Pimped serialized hashset 
/// 
/// *`set` The hashSet to serialized 
/// # Examples
///
/// ```rust
/// use std::collections::HashSet;
/// 
/// let hashset : HashSet<(u32,String)>;
/// 
/// hashset.insert((1,"localhost".to_string()));
/// hashset.insert((2,"127.0.0.1".to_string()));
/// 
/// assert_eq!(hashset_to_string(hashset), String::from("1,localhost;2,127.0.0.1"));
/// ```
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
/// # Examples
///
/// ```rust
/// use std::collections::HashSet;
/// 
/// let hashset_string : String = String::from("1,localhost;2,127.0.0.1");
///  
/// let hashset : HashSet<(u32,String)>;
/// 
/// hashset.insert((1,"localhost".to_string()));
/// hashset.insert((2,"127.0.0.1".to_string()));
/// 
/// assert_eq!(hashset_string,hashset);
/// ```
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
/// Size of a message
const TRAM_SIZE: usize = 500; 
/// Time in sec for nodes healthcheck
const REFRESH_TIME: u64 = 15;
/// Size of the block payload
const BLOCK_PAYLOAD_SIZE: usize = 5;
/// Difficulty of mining
const MINING_DIFFICULTY: usize = 1;

/// Util
/// Concat u8 array
/// * `first`
/// * `second`
/// # Example 
/// ```rust 
/// let first: &[u8] = String::from("Hello, ").as_bytes();
/// let second: &[u8] = String::from("World!").as_bytes();
/// asserteq!("Hello, World!",concat_u8(&first,&second));
/// ```
pub fn concat_u8(first: &[u8], second: &[u8]) -> Vec<u8> {
    [first, second].concat()
}


/// Add padding to the socket ip to allow fixed size data structure when sending data
pub fn encode_sockip(sockip: String) -> String {
    return format!("{:X<21}", sockip);
}

/// Remove the padding from the socket ip
pub fn decode_sockip(sockip: String) -> String {
    return str::replace(&sockip, "X", "");
}

/// Add padding to the ID to allow fixed sized data structure when sending an ID
pub fn encode_id(id: String) -> String {
    return format!("{:Y<10}", id);
}

/// Remove the padding from the ID field
pub fn decode_id(message: String) -> String {
    let id = str::replace(&message, "Y", "");
    return id;
}

/// Decode the message received
pub fn decode_message(msg : &[u8]) -> (Flag, String, String, String){
    println!("SSSDL111 .{:?}.", msg);
    let flag = Flag::from_u8(msg[0]); // get the flag
    let sockip_encoded = std::str::from_utf8(&msg[1..21]).unwrap();
    let id_encoded = std::str::from_utf8(&msg[22..31]).unwrap();
    let msg = std::str::from_utf8(&msg[32..]).unwrap();
    let sockip = decode_sockip(sockip_encoded.to_string());
    (flag, decode_sockip(sockip.to_string()), decode_id(id_encoded.to_string()), msg.to_string())
}

/// Encode the sended message
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

/// Used to create a new miner, then make it listen for incoming transactions
/// *`miner_type` - a char representing what kind of miner we need to create ('c' if we are creating a network, 'j' if we are joining an existing network)
/// *`socket` - the IP address on which the miner will listen for transactions
/// *`destination` - the IP address of a miner in the network we are joining, if `miner_type` == 'j'
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

/// This function is used when a new miner is joining an existing network, and needs a unique ID to be assigned to it
/// *`socket` - the miner whom we are joining
/// *`destination` - the joining miner IP address
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

/// Used by a joining miner to decode the unique ID he was given
/// *`stream` - a TCPStream instance of bytes which contains the ID
pub fn handle_id(mut stream: TcpStream) -> u32 {
    let mut data = [0 as u8; TRAM_SIZE];
    match stream.read(&mut data) {
        Ok(size) if size > 0 => {
            let tuple : (Flag, String, String, String) = decode_message(&data);
            let id_as_str : String = tuple.3.trim_matches(|c| c == char::from(0) || c == '\n').to_string();
            let id_as_str_decoded = decode_id(std::str::from_utf8(id_as_str.as_bytes()).unwrap().to_owned());
            let id = id_as_str_decoded.parse::<u32>().unwrap();
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

/// This struct is used to represent a Miner in our Blockchain Network
/// *`id` - the ID of our Miner, should be unique within the network
/// *`network` - a HashSet containing the IDs and IP addresses of every Miner inside the network
/// *`blocks` - a Vector containing the Blockchain
/// *`sockip` - the IP address on which our Miner will listen for incoming transactions
/// *`wallets` - a HashSet containing the IDs and IP addresses of every wallets binded to it
/// *`payload` - TBD
/// *`current_block_id` - TBD
pub struct Miner {
    pub id: u32, // Our ID
    pub network: HashSet<(u32, String)>, // The IDs and adresses of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub sockip: String,
    pub wallets: HashSet<(u32, String)>,
    pub payload: Vec<String>,
    pub current_block_id : u32,
}

impl Miner {

    /// CONSTRUCTOR
    /// `socket` - an ip:port string representing where is the Miner listening
    /// Returns a new Miner with a TcpListener that listens to the given ip:port
    pub fn new (id: u32, socket: String) -> Self {
        return Miner {
            id: id,
            network: HashSet::new(),
            blocks: Vec::new(),
            sockip: socket.to_string(),
            wallets: HashSet::new(),
            payload: Vec::new(),
            current_block_id: 0,
        }        
    }

    /// Getter for the `id` attribute of a Miner
    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// Function used to print the `network` attribute of a Miner
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
                let m: &[u8] = &encode_message(flag, self.sockip.to_string(), self.id.to_string(), message.to_string());
                println!("Byte message: {:?}",&m);
                match stream.write(m) {
                    Ok(_) => println!("Message writen in buffer"),
                    Err(e) => println!("Error during writing: {}", e.to_string()),
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

    /// This function is used by a Miner to Broadcast an incoming message to the whole network
    /// *`message` - the message to be broadcasted
    /// *`flag` - the flag associated with the message
    /// *`sender` - the IP address of the miner which sended the original message
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

    /// Used by an existing Miner when a new miner joins the network and asks for it's unique ID
    /// Returns an integer which will be assigned as the joining Miner's ID
    pub fn retrieve_next_id(&self) -> u32 {
        let mut max_id = &self.id;
        for (id, _) in &self.network {
            if id > max_id {
                max_id = id;
            }
        }
        let new_id = max_id+1;
        return new_id.to_owned();
    }

    /// Used by an existing Miner when a new wallet is binded to it and asks for it's unique ID
    /// Returns an integer which will be assigned as the new wallet's ID
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

    /// This function handles the incoming requests/messages sended by other entities
    /// *`stream` - a TCPStream instance which contains the data that was received
    /// The function decodes the incoming message, then act accordingly
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
                        // When a miner connects to the network, add him to our HashSet then broadcast the connection message to the network
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
                        // If a miner disconnects from the network, remove him from our HashSet and broadcast the disconnection to the Network
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

                        self.network = self.network.union(&network).into_iter().cloned().collect::<HashSet<_>>().clone();
                        println!("New network: ");
                        for (i,e) in &self.network {
                            println!("{}, {}",i,e);
                        }
                        // self.broadcast(&message, flag);
                        self.refresh_nodes_status();
                    }
                    Flag::RequireID => {
                        println!("RequireID Flag received");
                        let next_id = self.retrieve_next_id().to_string();
                        match self.send_message(&sender_sockip, &next_id, Flag::GiveID) {
                            Ok(_) => println!("ID correctement envoyé"),
                            Err(e) => println!("Err: {}", e),
                        }
                    }
                    Flag::Check => {
                        println!("Check Flag received");
                        self.send_message(&sender_sockip, &String::from(""), Flag::Ack);
                    }
                    Flag::Ack => {
                        println!("Ack Flag received: Do nothing");     
                    }
                    Flag::Block => {
                        println!("Block received");
                        self.refresh_nodes_status();
                        // Check block
                        // if &self.check_block(block::Block::from(message)){
                            // forward block
                            // &self.broadcast_to_network(message, Flag::Block,sender_sockip);
                            // add to blockchain
                            // Remove transaction already done
                        // } else {
                            // Invalid block
                        //}   
                    }
                    Flag::Transaction => {
                        println!("Transaction Flag received");
                        self.refresh_nodes_status();
                        // Je regarde si je l'ai deja
                        let transaction : String = message.trim_matches(|c| c == char::from(0) || c == '\n').to_string();
                        
                        if !&self.payload.contains(&transaction) {
                            self.payload.push(transaction.to_string());
                            &self.broadcast_to_network(&transaction, Flag::Transaction, sender_sockip.to_owned());
                        }
                        // TODO: Handle \u{0} char
                        println!("{:?}",&self.payload);

                        // Check payload size
                        if &self.payload.len() == &BLOCK_PAYLOAD_SIZE {
                            // &self.broadcast_to_network(&self.payload.join(";").to_string(), Flag::MineTransaction, sender_sockip);
                            let payload = &self.payload.join("$").to_string();
                            println!("Payload to mine: {}", payload);
                            let mined_block : block::Block = self.hash_block(payload.to_string());
                            println!("Mined block: {:?}", &mined_block);
                            // Ajoute a la blockchain
                            if self.blocks.len() as u32 == mined_block.index {
                                &self.blocks.push(block::Block::from_str(&mined_block.to_string()).unwrap());
                                &self.broadcast_to_network(&mined_block.to_string(), Flag::Block, sender_sockip.to_owned());
                                self.payload = Vec::new();
                                println!("Chain: {:?}", &self.blocks);
                            }
                            // TODO: Thread
                                    // Lock var
                            // let m = Mutex::new(0);
                            // // let m = Mutex::new(self.current_block_id);
                            // if let Ok(mut x) = m.lock() {
                            //     println!("Thread 1 acquired lock");
                            //     self.current_block_id += 1;
                            // };   
                            // let new_id = self.current_block_id;
                            // println!("{}",new_id);  
                            // std::mem::drop(m); // Unlock
                            // thread::scope(|s| {
                            //     s.spawn(move |_| {
                            //         let mined_block : block::Block = self.hash_block(payload.to_string());
                            //         println!("Mined block: {:?}", &mined_block);
                            //         // Ajoute a la blockchain
                            //         if self.blocks.len() as u32 == mined_block.index {
                            //             &self.blocks.push(&mined_block);
                            //             &self.broadcast_to_network(&mined_block.to_string(), Flag::Block, sender_sockip.to_owned());
                            //             println!("Chain: {:?}", &self.blocks);
                            //         }
                            //     });
                            // });
                            
                        }    
                    }
                    Flag::MineTransaction => {
                        // Verif if transaction are in payload

                        // If true
                            // 
                    }
                    Flag::OkMineTransaction => {
                        // Verif if transaction are in payload
                        // If true
                            // 
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
                    Flag::RequireBlockchain => {
                        println!("Required Blockchain hashes");
                        for block in self.blocks.iter() {
                            self.send_message(&sender_sockip, &block.to_string(), Flag::SendBlockchain);
                        }
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
        let mut init_time = Instant::now();
        let listener = TcpListener::bind(&self.sockip).unwrap();
        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", &stream.peer_addr().unwrap());  
                    &self.handle_client(stream);
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
            println!("Time spend: {}",&init_time.elapsed().as_secs());
            if init_time.elapsed().as_secs() >= REFRESH_TIME {
                println!("Check time spend");
                init_time = Instant::now();
                &self.refresh_nodes_status();
            }
            self.display_network();
        }
        // close the socket server
        println!("Closing listener");
        drop(listener);
    }

    /// Used by Miners to ask the Miner to which it is connecting to give us our ID
    /// *`socket` - our IP address
    /// *`destination` - the IP address of the Miner we are asking for an ID
    pub fn ask_for_id(socket: &String, destination: &String) -> u32 {
        println!("Asking {} for id...", &destination);
        let listener = TcpListener::bind(&socket).unwrap();
        let mut id: u32 = 0;
        // Ask the ID
        if let Ok(mut stream) = TcpStream::connect(&destination) {
            let m: &[u8] = &encode_message(Flag::RequireID, socket.to_string(), "".to_string(), "".to_string());
            match stream.write(m) {
                Ok(_) => { println!("Asked for id"); }
                Err(e) => { println!("Error: {}", e); }
            }
            println!("Message sended");
        }
        // Handle the response
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

    /// Used by wallets to ask the Miner to which it is connecting to give us our ID
    /// *`socket` - our IP address
    /// *`destination` - the IP address of the Miner we are asking for an ID
    pub fn ask_miner_for_wallet_id(socket: &String, miner: &String) -> u32 {
        println!("Asking {} for wallet ID", &miner);
        let listener = TcpListener::bind(&socket).unwrap();
        let mut id: u32 = 0;
        // Ask the ID
        if let Ok(mut stream) = TcpStream::connect(&miner) {
            let m: &[u8] = &encode_message(Flag::RequireWalletID, socket.to_string(), "".to_string(), "".to_string());
            match stream.write(m) {
                Ok(_) => { println!("Asked for id"); }
                Err(e) => { println!("Error: {}", e); }
            }
            println!("Message sended, wallet added to miner's wallet list");
        }
        // Handle the response
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

    /// Simple function to add a new wallet to our Miner's wallet list
    /// *`peer_id` - the ID of the new wallet
    /// *`peer_addr` - the IP address of the new wallet
    pub fn add_to_wallets(&mut self, peer_id: u32, peer_addr: String) -> bool {
        self.wallets.insert((peer_id, peer_addr))
    }    
    /// Function to refresh all nodes status and remove those are not accessible
    pub fn refresh_nodes_status(&mut self){
        println!("Refresh node status");
        let nodes: &HashSet<(u32,String)> = &self.network.to_owned();
        for (id,addr) in nodes {
            if id != &self.id {
                println!("\n node: {}",&id);
                &self.health_check(&addr, &id);
            }
        }
    }

    /// health_check
    /// ping the destination. If it doesn't respond ok or at time, removing the destination node from network
    /// 
    pub fn health_check(&mut self, destination: &String, id: &u32) -> Result<u8,&'static str>{
        let result = &self.send_message(destination, &String::from(""), Flag::Check);  
         
        match result{
            Ok(code) => { println!("Ok healthcheck: {}", code); }
            Err(error) => {
                println!("HealthCheck failed: {}", &error);
                println!("Removing node: {},{}", &id, &destination);
                &self.remove_from_network(*id, String::from(destination));
                println!("node removed");
            }

        }
        Ok(0)
    }

    /// Function to check if the received block is valid
    /// 
    /// *`block` the received block
    /// Return true if the block is valid, false else
    pub fn check_block(&self, block: block::Block) -> bool {
        // Verif hash 
        let last_block: &block::Block = &self.blocks.last().unwrap();
        println!("Last block: {:?}", last_block);
        println!("Received block: {:?}", block);
        if &last_block.index == &(block.index+1) && &last_block.payload == &block.payload && &last_block.hash == &last_block.prev_hash {
            // hash
            return true
        } else {
            return false
        }
    }

    /// Function to mine block
    /// *`transaction` List of transactions to mine
    /// *`MINING_DIFFICULTY` use 
    /// *Return* the mined block
    pub fn hash_block(&self, transactions: String) -> block::Block{

        let start = SystemTime::now();
        let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
        let timestamp_ms = since_the_epoch.as_millis();
        let nonce: u64 = 5;
        
        let index_: u32;
        let timestamp_: u128 = timestamp_ms;
        let payload_: String = transactions;
        let nonce_: u64 = nonce;
        let previous_hash_: String;
        
        let last_block: &block::Block;
        match self.blocks.last() {
            Some(block) => {
                last_block = block;
                index_ = last_block.index + 1;
                previous_hash_ = hex::encode(&last_block.hash);
            }
            _ => {
                // Genesis
                index_ = 0;
                let mut sha256 = Sha256::new();
                sha256.update("first_block");
                let first_hash: String = format!("{:x}", sha256.finalize());
                println!("first_hash: {}",first_hash);
                previous_hash_ = first_hash;
            }
        } 
        
        let mut to_hash;
        let mut sha256;
        let mut hash: String = String::from("");
        for nonce_i in 0..(u64::max_value()) {
            to_hash = index_.to_string() + &payload_ + &timestamp_.to_string() + &nonce_i.to_string() + &previous_hash_;
            
            sha256 = Sha256::new();
            sha256.update(to_hash);
            hash = format!("{:x}", sha256.finalize());
            
            
            if hash.starts_with(&"0".repeat(MINING_DIFFICULTY)) {
                println!("new_hash: {}",&hash);
                return block::Block{
                    index: index_, 
                    payload: payload_, 
                    timestamp: timestamp_, 
                    nonce: nonce_i, 
                    prev_hash: previous_hash_.as_bytes().to_vec(), 
                    hash: hash.as_bytes().to_vec()
                };
            }
        }
        /// Dead code, normaly upside code find hash before u64::max_value iteration.
        return block::Block{
            index: index_, 
            payload: payload_, 
            timestamp: timestamp_, 
            nonce: nonce_, 
            prev_hash: previous_hash_.as_bytes().to_vec(), 
            hash: hash.as_bytes().to_vec()
        };
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
