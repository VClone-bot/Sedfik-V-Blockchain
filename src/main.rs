use blocklib::*;
use std::env;
mod miner;

fn main() {
    let block = Block::new(0, "Premier bloc".to_owned(), 0, 0, vec![0; 32]);
    println!("{:?}", &block);


    let args: Vec<String> = env::args().collect();
    
    // Si ip null alors localhost 
    
    let role = &args[1]; // --create / --join

    let mut address = "localhost:3000";
    
    if args.len() >= 3 {
     
        address = &args[2]; // ip:port / localhost:3000
    }

    match role.as_str() {
        "--create" => miner::create_miner(), // Create root
        // "--join" => "", // Join
        _ => println!("Not a valid argument."),
    }

    println!("You role: {}, addr: {}", role, address);
   
}
