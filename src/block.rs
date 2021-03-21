use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;
use std::num::ParseIntError;

/** Bloc: composants de la BlockChain
 * Composants d'un bloc
 * - Index: la position de ce bloc dans la chaîne
 * - Payload: les infos/événéments qui ont eu lieu dans le bloc
 * - Timestamp: pour avoir une notion de temps
 * - Nonce: nombre utilisé pour calculer le Proof of Work
 * - Previous block hash: L'empreinte cryptographique du bloc précédent
 * - Hash: l'empreinte cryptographique de toutes les données ci-dessus, concatanées ensemble
 */

pub struct Block {
    pub index: u32,
    pub payload: String,
    pub timestamp: u128,
    pub nonce: u64,
    pub prev_hash: Vec<u8>,
    pub hash: Vec<u8>,
}

impl Block {
    pub fn new (index: u32, payload: String, timestamp: u128, nonce: u64, prev_hash: Vec<u8>) -> Self {
        return Block {
            index,
            payload,
            timestamp, 
            nonce,
            prev_hash,
            hash: vec![0; 16],
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id:{};prev_hash:{};timestamp:{};payload:{};nonce:{};hash:{}",
            &self.index,
            &hex::encode(&self.prev_hash),
            &self.timestamp,
            &self.payload,
            &self.nonce,
            &hex::encode(&self.hash),
        )
    }
}

impl Debug for Block {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "id:{};prev_hash:{};timestamp:{};payload:{};nonce:{};hash:{}",
            &self.index,
            &hex::encode(&self.prev_hash),
            &self.timestamp,
            &self.payload,
            &self.nonce,
            &hex::encode(&self.hash),
        )
    }
}

impl FromStr for Block {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let block: HashMap<String, String> = s.split(';')
        .map(|kv| kv.split(':').collect::<Vec<&str>>())
        .map(|vec| {
            (vec[0].to_string(), vec[1].to_string())
        })
        .collect();
        Ok(Block { 
            index: block.get("id").unwrap().parse::<u32>().unwrap(), 
            payload: block.get("payload").unwrap().to_string(),
            timestamp: block.get("timestamp").unwrap().parse::<u128>().unwrap(), 
            nonce: block.get("nonce").unwrap().parse::<u64>().unwrap(), 
            prev_hash: block.get("prev_hash").unwrap().as_bytes().to_vec(),
            hash:  block.get("hash").unwrap().as_bytes().to_vec(),
     })
    }
}