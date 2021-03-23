use std::str::FromStr;
use std::env;
use clap::{Arg, App, SubCommand};

mod miner;
mod wallet;

mod block;
fn main() {

    // let block = block::Block::from_str(&block::Block::new(0, "Premier bloc".to_owned(), 0, 0, vec![0; 32]).to_string());
    // println!("{:?}", &block);
    //Pour afficher le --help
    let matches = App::new("Blockchain")
        .version("0.1.0")
        .author("Team ViVaSi <vivasi@u.fr>")
        .about("Projet Blockchain - UE15 - M2ISD")
        .arg(Arg::with_name("create")
                 .short("c")
                 .long("create")
                 .value_name("HOST:PORT")
                 .takes_value(true)
                 .help("Create a new miner on the specified host:port"))
        .arg(Arg::with_name("join")
                 .short("j")
                 .long("join")
                 .value_names(&["HOST:PORT" ,"HOST:PORT"])
                 .takes_value(true)
                 .help("Join miner on the specified host:port"))
        .arg(Arg::with_name("wallet")
                .short("w")
                .long("wallet")
                .value_names(&["HOST:PORT", "HOST:PORT"])
                .takes_value(true)
                .help("Join wallet to a specified miner on host:port"))
        .get_matches();


    let create = matches.value_of("create").unwrap_or("");
    println!("{}",create);
    
    let join = matches.value_of("join").unwrap_or("");
    println!("{}",join);

    let wallet = matches.value_of("wallet").unwrap_or("");
    println!("{}",wallet);

    let args: Vec<String> = env::args().collect();
    
    // On vérifie les arguments
    if args.len() == 1 {
        println!("miner: no operation specified");
        return ();
    } else if args.len() == 2 {
        println!("miner: must specify an IP and a port to join a network");
        return ();
    } else if args.len() > 4 {
        println!("miner: too many arguments specified");
        return ();
    }

    let role; // {--create, -c} pour créer un réseau, {--join, -j} pour en rejoindre un existant
    let socket = &args[2]; // L'ip:port du socket sur lequel le miner va écouter
    let mut address = &args[2];
    
    if &args[1] == "-c" || &args[1] == "--create" {
        role = "creator";
    } else if &args[1] == "-j" || &args[1] == "--join" {
        role = "joiner";
    } else if &args[1] == "-w" || &args[1] == "--wallet" {
        role = "wallet";
    } else {
        println!("miner: operation not recognized");
        return ();
    }

    if role == "joiner" {
        address = &args[3];
        miner::create_miner('j',socket.to_string(), address.to_string());
    }

    if role == "creator" {
        miner::create_miner('c',socket.to_string(), String::new());
    } else if role == "joiner" {
        miner::create_miner('j',socket.to_string(), address.to_string());
    }

    if role == "wallet" {
        address = &args[3];
        println!("wallet : {} ; {}",socket.to_string(), address.to_string());
        wallet::create_wallet(socket.to_string(), address.to_string());
    }

    return ();
}
