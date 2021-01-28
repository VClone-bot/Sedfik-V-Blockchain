use std::fmt::{self, Debug, Formatter};

/** Bloc: composants de la BlockChain
 * Composants d'un bloc
 * - Index: la position de ce bloc dans la chaîne
 * - Payload: les infos/événéments qui ont eu lieu dans le bloc
 * - Timestamp: pour avoir une notion de temps
 * - Nonce: nombre utilisé pour calculer le Proof of Work
 * - Previous block hash: L'empreinte cryptographique du bloc précédent
 * - Hash: l'empreinte cryptographique the toutes les données ci-dessus, concatanées ensemble
 */

pub struct Block {
    pub index: u32,
    pub payload: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub prev_hash: Vec<u8>,
    pub hash: Vec<u8>,
}

impl Block {
    pub fn new (index: u32, payload: String, timestamp: u64, nonce: u64, prev_hash: Vec<u8>) -> Self {
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

impl Debug for Block {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Block[{}]: {} at: {} with: {}",
            &self.index,
            &hex::encode(&self.hash),
            &self.timestamp,
            &self.payload,
        )
    }
}