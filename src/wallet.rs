use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use crossbeam_utils::thread;
use std::collections::HashSet;

#[path="./miner.rs"]
mod miner;

pub struct Wallet {
    pub id: u32, // Our ID
    pub miner: &Miner,
    pub socket: String,
}

pub fn create_wallet(socket: String, miner: String) ->  {
    println!("Wallet creation...");
    //Ask our miner what our ID is
    let new_id: u32 = miner::ask_miner_for_wallet_id(&socket, &miner);
    let wallet = Wallet::new(socket, miner, new_id);
}

impl Wallet {

    /// CTOR
    /// `miner` - the miner to which that wallet is tied04
    pub fn new(socket: String, miner: String, id: u32) -> Self {
        return Wallet {
            socket: socket,
            miner: miner,
            id: id,
        }
    }

    /// Function to send a message
    /// * `stream` - Tcp Stream.
    /// * `message` - The message to send.
    pub fn send_message(&self, destination: &String, message: &String, flag: Flag) -> Result<u8, &'static str> {
        let f = flag as u8;
        println!("Sending message: {} \nTo: {} .. {} \nWith Flag: {}", &message, &destination, &destination.chars().count(), &f);
        match TcpStream::connect(&destination) {
            Ok(mut stream) => {
                println!("Connection established.");
                let m: &[u8] = &encode_message(flag, self.sockip.to_string(), self.id.to_string(), message.to_string());
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
}

impl Debug for Miner {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Wallet[{}]",
            &self.id,
        )  
    }
}