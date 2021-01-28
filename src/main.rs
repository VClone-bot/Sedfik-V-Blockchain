use std::env;

mod miner;

fn print_help() {
    println!("Lol");
}

fn main() {

    //let block = Block::new(0, "Premier bloc".to_owned(), 0, 0, vec![0; 32]);
    //println!("{:?}", &block);

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

    let mut role = ""; // {--create, -c} pour créer un réseau, {--join, -j} pour en rejoindre un existant
    let mut socket = &args[2]; // L'ip:port du socket sur lequel le miner va écouter
    
    if &args[1] == "-c" || &args[1] == "--create" {
        role = "creator";
    } else if &args[1] == "-j" || &args[1] == "--join" {
        role = "joiner";
    } else {
        println!("miner: operation not recognized");
        return ();
    }

    if role == "creator" {
        let mut address = &args[3]; // L'ip:port à laquelle le miner va se connecter 
    }

    //si role == -c -> créer réseau
    //si role == -j & ip donnée -> rejoindre le réseau
    //sinon écrire une erreur et renvoyer unit
    if role == "-c" || role == "--create" {
        let test: Vec<u32> = (0..10).collect();
    } else if role == "-j" || role == "--join" {
        let test: Vec<u32> = (0..10).collect();
    } else if role == "--help" {
        print_help();
        return ();
    } else {
        println!("miner: the command {} is unknown, please see miner --help", role);
        return ();
    }

    

}
