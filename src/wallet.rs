use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Write, Read};
use crate::miner::Miner;
use std::collections::HashSet;
use std::process::Command;

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
            _ => panic!("Unknown value: {}", value),
        }
    }
}

/// Used to represent the commands that the user can send to the wallet via the standard input
/// 
#[derive(Copy, Clone)]
pub enum UserCommand {
    Send,
    Check,
    Exit,
}

/// This function converts the String text passed in the console to UserCommands
/// 
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

/// Util
/// Concat u8 array
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
pub fn decode_message(message: String) -> String {
    return str::replace(&message, "Y", "");
}

/// This struct represent the wallets
/// *`id` - an integer than should be unique to each wallet within a miner's wallet list
/// *`miner` - the IP address of the miner to which the wallet is binded
/// *`socket` - the IP address on which the wallet listens for incoming messages
pub struct Wallet {
    pub id: u32, // Our ID
    pub miner: String,
    pub socket: String,
}

/// This function is used to send a message to another entity on the network
/// *`flag` - a Flag which represents what kind of request/message we are sending
/// *`sockip` - the IP address of the recipient of the message
/// *`id` - only used when sending an ID value, otherwise it should be left blank
/// *`msg` - the message sended to the recipient
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

/// This function creates a wallet and make it listen for the user input
/// *`socket` - the IP address on which the wallet is listening
/// *`miner` - the IP address of the miner it is binded to
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
    /// *`socket` - the IP address where the wallet listens
    /// *`miner` - the IP address miner to which that wallet is tied
    /// *`id` - the unique ID that must be assigned to this wallet
    /// Returns a new wallet with parameters initialized as given
    pub fn new(socket: String, miner: String, id: u32) -> Self {
        return Wallet {
            socket: socket,
            miner: miner,
            id: id,
        }
    }

    /// This function allows our wallet to listen to the commands given by the user on std input
    /// It checks for validity of the input, and act accordingly
    pub fn listen_for_user_input(&self) {
        let stdin = io::stdin();
        loop {
            let mut buffer = String::new();
            println!("Ready for input...");
            stdin.read_line(&mut buffer);
            let splitted: Vec<&str> = buffer.split(" ").collect();
            let command = UserCommand::from_string(splitted[0].to_string());

            //On gère l'input de l'utilisateur
            match command {
                UserCommand::Send => {
                    let message = splitted[1].to_string();
                    println!("Response: {}\n", self.handle_user_input(command, self.miner.to_string(), message.to_string()));
                }
                UserCommand::Check => {
                    println!("Response: {}\n", self.handle_user_input(command, "".to_string(), "".to_string()));
                }
                UserCommand::Exit => {
                    println!("Response: {}\n", "Ok".to_string());
                    break;
                }
                _ => { () }
            }
        }

        println!("Disconnecting Wallet");
        return ();
    }

    /// Performs the action which the user gave as input
    pub fn handle_user_input(&self, command: UserCommand, target: String, message: String) -> String {
        return match command {
            UserCommand::Send => {
                println!("Sending message to Miner...");
                let listener = TcpListener::bind(&self.socket).unwrap();
                if let Ok(mut stream) = TcpStream::connect(&self.miner) {
                    let m: &[u8] = &encode_message(Flag::Transaction, self.socket.to_string(), self.id.to_string(), message.to_string());
                    match stream.write(m) {
                        Ok(_) => { println!("Message {} sended to {}", message.to_string(), target.to_string()); }
                        Err(e) => { println!("Error: {}", e); }
                    }
                    println!("Message sended");
                }

                
                return "".to_string();
            }
            UserCommand::Check => {
                //let response = self.send_message(miner.to_string(), "".to_string(), Flag::Check);
                return "Check ok".to_string();
            }
            _ => "Unknown command".to_string()
        }
    }

    /// This handles incoming message, by decoding them and transforming them into usable data
    /// *`stream` - is a TcpStream instance containing the bytes that we received
    pub fn handle_message(&self, mut stream: TcpStream) -> String {
        let mut data = [0 as u8; 50];
        match stream.read(&mut data) {
            Ok(size) if size > 0 => {
                let response_decoded = decode_message(std::str::from_utf8(&data[32..size]).unwrap().to_owned());
                return response_decoded;
            },
            Ok(_) => { println!("No message received");},
            Err(e) => {
                println!("Error occured, closing connection: {}", e);
                stream.shutdown(Shutdown::Both).unwrap();
                return "Error".to_string();
            }
        }
        {}
        return "Error".to_string();
    }

    /// Function to send a message to another entity on the network
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