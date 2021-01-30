# Projet Blockchain - Master ISD S4

 BARTHET Virgile, RAMOS Valentin & ROUFF Simon

## Compilation:
`cargo build`

## Fonctionnement:
Pour créer un mineur, `./target/debug/blockchain -c adresseIp:Port`

Pour créer y connecter un autre mineur: `./target/debug/blockchain -c adresseIpDuPremierMineur:PortDuPremierMineur adresseIpDuJoiner:PortDuJoiner`