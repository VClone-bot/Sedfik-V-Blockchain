# Projet Blockchain - Master ISD S4

**BARTHET Virgile, RAMOS Valentin, ROUFF Simon** :pencil: 

[Lien du repo Github](https://github.com/VClone-bot/Sedfik-V-Blockchain/tree/master) 

[Lien de la documentation technique (page WEB)](https://vclone-bot.github.io/Sedfik-V-Blockchain/blockchain/index.html)

## Présentation du projet

Ce projet a été intégralement écrit en Rust. Nous avons choisi ce langage car c'est un langage de programmation de bas-niveau et que surtout nous voulions l'apprendre.
Ce projet consiste en la création d'une blockchain, c'est à dire programmer des mineurs, des wallets, pouvoir miner des blocs, et s'échanger des transactions. Il a été réalisé intégralement *from scratch*, en utilisant juste les *crates* suivants:
* `hex`
* `rand`
* `crossbeam-utils`
* `sha2`
* `clap`

On a choisi de communiquer dans notre blockchain avec une grande quantité de **flag** que vous trouverez décrit ci-dessous.


## Manuel

Installation de Rust:
* Unix: ```curl https://sh.rustup.rs -sSf | sh```  [page d'installation officielle](https://www.rust-lang.org/tools/install)
* Windows: Télécharger et installer [rustup-init.exe](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)
* Autre distribution/Aide: [Other installation methods](https://forge.rust-lang.org/infra/other-installation-methods.html)

Compilation:
``` bash = 
cargo build
```
Les bibliothèques nécessaires seront **automatiquement** installées par ```rustc```.

Commandes disponibles:
```bash=
./target/debug/blockchain --help
```
### Commandes Mineur
Créer un mineur:
```bash=
./target/debug/blockchain -c adresseIp:Port
```

Connecter un mineur:
```bash=
./target/debug/blockchain -j adresseIP1erMineur:port1erMineur adresseIPCible:portCible
```
### Commandes Wallet
Les commandes pouvant être utilisées dans la console du Wallet sont les suivantes :

```bash=
Send message
```

*avec *message* le message à envoyer.


## Structures
### Bloc

Un bloc est composé:
 * Index: la position de ce bloc dans la chaîne
 * Payload: les infos/événéments qui ont eu lieu dans le bloc
 * Timestamp: pour avoir une notion de temps (em ms depuis le 01/01/1970)
 * Nonce: nombre utilisé pour calculer le Proof of Work
 * Previous block hash: L'empreinte cryptographique du bloc précédent
 * Hash: l'empreinte cryptographique de toutes les données ci-dessus, concatanées ensemble

Les index, timestamp et nonce sont des entiers.
Le payload est une liste de messages, donc de String
Le previous block hash et le hash sont des hash, stockés en String. Ce hash est obtenu avec l'algorithme `SHA-256` en concaténant l'index, le payload, le timestamp, le nonce et le previous hash.
### Mineur

Un mineur est composé:
* `id`: l'id du mineur, qui est unique dans le réseau. En cas de déconnexion, aucun autre mineur peut prendre son id, sauf si le mineur a été enlevé du réseau.
* `network`: une HashSet contenant les id et les adresses IP de tous les mineurs dans le réseau.
* `blocks`: un vecteur qui contient la liste de tous les blocks, autrement dit l'entièreté de la blockchain.
* `sockip`: l'adresse IP que le mineur va écouter pour recevoir les transactions
* `wallets`: une HashSet contenant la liste de tous les wallets connectés à ce mineur
* `payload`: les informations de la blockchain
* `current_block_id`: l'ID du block en cours de minage
### Wallet

Un wallet est composé de 
* `socket`: l'adresse IP sur laquelle le wallet écoute
* `miner`: l'adresse IP du mineur auquel le wallet est associé
* `id`: l'id du mineur

### Transaction

On a choisi pour représenter les transactions d'utiliser des messages, dont le contenu est détaillé plus bas :arrow_double_down: 

#### Structure d'un message :email: 

|Nom|Taille (octet)|Description|
|--|--|--|
|Flag|1|Flag du message|
|sockip|21|Adresse Ip de l'émetteur|
|id|10|Id du mineur (noeud) émetteur|
|message|TRAM_SIZE - 32| Message transmis|
|**Total**|500|

#### Flag utilisés :triangular_flag_on_post: 

Ces flags sont utilisés pour la communication entre les différents éléments de la blockchain à travers le réseau. C'est un type énuméré, ce qui a pour avantage d'être très lisible lors de la programmation et lors de la lecture du code. C'est aussi un type très flexible, ce qui nous permet d'ajouter des éléments au fur et à mesure du rajout de fonctionnalités.

|Nom|Description|
|--|--|
|Ok|Recu lors de la connexion avec un mineur afin de signifier qu'il est bien connecté au réseau|
|Connect|Demande de connexion d'un mineur|
|Disconnect|Flag reçu à la deconnexion d'un mineur|
|RequireID|Utilisé pour demander le prochain id à utiliser lors de la création d'un mineur|
|BroadcastConnect|Broadcast la connexion d'un mineur|
|BroadcastDisconnect|Broadcast la deconnexion d'un mineur|
|Check|Simple ping recu lors du healthcheck|
|Ack|Retour du healthcheck|
|Block|Transmition d'un block|
|Transaction|Transmition de transaction|
|MineTransaction|Flag d'information pour lancer le minage d'un block|
|OkMineTransaction|Flag Ack pour MineTransaction|
|RequireWalletID|Demande le prochain Id pour un Wallet|
|RequireBlockchain|L'arbre de Merkle demande au mineur la blockchain pour la vérifier|
|SendBlockchain| L'arbre de Merkle envoie la blockchain au mineur une fois vérifiée|

## Checklist :pencil: 
- [X] Mineur
    - [x] créer un mineur
    - [x] connecter un mineur au réseau
    - [x] Gérer la déconnexion d'un mineur 
- [x] Wallet 
    - [x] Créer des wallets
    - [x] Ajouter un wallet à un mineur existant
    - [x] Envoyer des messages
    - [x] Check Block
    - [x] Send Block 
- [ ] Minage 
    - [x] Algo de minage
    - [x] Vérifier le calcul
- [x] Arbre de Merkel
    - [x] Vérifier si une transaction est faite (l'intégrité)


