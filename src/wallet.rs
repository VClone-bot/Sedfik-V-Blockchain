use std::net::{TcpStream};
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Write, Read};
use crate::miner::Miner;
use std::collections::HashSet;
use std::process::Command;
//use crossbeam_utils::thread;

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
            _ => panic!("Unknown value: {}", value),
        }
    }
}

#[derive(Copy, Clone)]
pub enum UserCommand {
    Send,
    Check,
    Exit,
}

impl UserCommand {
    fn from_string(value: String) -> UserCommand {
        let v = value.trim();
        let s_value: &str = &v[..];  // take a full slice of the string
        match s_value {
            "Send" => UserCommand::Send,
            "Check" => UserCommand::Check,
            "Exit" => UserCommand::Exit,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

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

pub struct Wallet {
    pub id: u32, // Our ID
    pub miner: String,
    pub socket: String,
}

pub fn encode_message(flag : Flag, sockip : String, id : String, msg : String) -> Vec<u8>{
    println!("\nEncoding message");
    let flag_convert: &[u8] = &[flag as u8];
    let sockip_convert : String = encode_sockip(sockip);
    let id_convert : String = encode_id(id);
    println!("\tmessage to encode: {}", &msg);
    let msg_convert : &[u8] = msg.as_bytes();
    println!("\tmessage encoded: {:?}", &msg_convert);
    concat_u8(flag_convert, &concat_u8(sockip_convert.as_bytes(), &concat_u8(id_convert.as_bytes(), msg_convert)))
}

pub fn create_wallet(socket: String, miner: String) {
    println!("Wallet creation...");
    //Ask our miner what our ID is and create the wallet with given id
    let new_id: u32 = Miner::ask_miner_for_wallet_id(&socket, &miner);
    let wallet = Wallet::new(socket, miner, new_id);

    //Listen for user input
    wallet.listen_for_user_input();
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

    pub fn listen_for_user_input(&self) {
        let mut stdin = io::stdin();
        loop {
            let mut buffer = String::new();
            println!("Ready for input...");
            stdin.read_line(&mut buffer);
            let command = UserCommand::from_string(buffer);

            //On gère l'input de l'utilisateur
            let response = self.handle_user_input(command);
            println!("Response: {}\n", response);
            if(response == "Exit") { 
                break;
            }
        }

        println!("Disconnecting Wallet");
        return();
    }

    pub fn handle_user_input(&self, command: UserCommand) -> String {
        return match command {
            UserCommand::Send => {
                "Sending".to_string()
            }
            UserCommand::Check => {
                "Checking".to_string()
            }
            UserCommand::Exit => {
                "Exit".to_string()
            }
            _ => "Unknown command".to_string()
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
                let m: &[u8] = &encode_message(flag, self.socket.to_string(), self.id.to_string(), message.to_string());
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

impl Debug for Wallet {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Wallet[{}]",
            &self.id,
        )
    }
}