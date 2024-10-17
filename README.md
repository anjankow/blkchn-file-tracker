# Deploy solana program locally
From https://solana.com/developers/guides/getstarted/local-rust-hello-world

```sh
cd solana_program/
```

## Build
```sh
cargo build-sbf
```

## Configure Solana CLI
Configure your Solana CLI to use your localhost validator for all your future terminal commands and Solana program deployment:
```sh
solana config set --url localhost
```

After setting a cluster target, any future subcommands will send/receive information from that cluster. To set it to devnet:
```sh
solana config set --url https://api.devnet.solana.com
```

## Run test validator
Run a local validator for tests:
```sh
solana-test-validator
```

### Get configs
```sh
solana config get
```
Outputs:
```plain
Config File: /root/.config/solana/cli/config.yml
RPC URL: http://localhost:8899 
WebSocket URL: ws://localhost:8900/ (computed)
Keypair Path: /root/.config/solana/id.json 
Commitment: confirmed 
```

### First setup
```sh
solana-keygen new -o ${HOME}/.config/solana/id.json
```

### Money 💸
```sh
solana airdrop 500
```

## Deploy
```sh
solana program deploy ./target/deploy/file_event_tracker.so
```

### List all programs deployed by an address
```sh
solana program show --programs
```
Outputs:
```plain
Program Id                                   | Slot      | Authority                                    | Balance
EqWRXakMW7rJvT4dMUd1uUdNyKbbvEP48kK9NCevfAJ4 | 1768      | E617pwHkquBHUPAqcThYYmw1Wbhcwy1vq8V4vnPpAfMY | 0.4321116 SOL
```

## Watch
https://explorer.solana.com/?cluster=custom


