# Projet Blockchain - Master ISD S4

 BARTHET Virgile, RAMOS Valentin & ROUFF Simon

## Compilation:
`cargo build`

## Fonctionnement:
Pour créer un mineur, `./target/debug/blockchain -c adresseIp:Port`

Pour créer y connecter un autre mineur: `./target/debug/blockchain -c adresseIP:port adresseIP:port` 
Le premier couple IP:port est celui sur lequel on souhaite faire écouter le mineur, le deuxième correspond à celui du mineur déjà existant que l'on souhaite rejoindre
