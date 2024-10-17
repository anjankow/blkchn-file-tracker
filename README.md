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
https://docs.solanalabs.com/cli/examples/deploy-a-program

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

### List all buffers
Program Buffer - a temporary account that stores byte code while a program is being actively deployed or upgraded. Once the process is complete, the data is transferred to the Program Executable Data Account and the buffer account is closed.
```sh
solana program show --buffers
```
```plain
Buffer Address                               | Authority                                    | Balance
65imwR88tsyxgwXcfnE27ENpzkbbrKkhUktU5mZJT2Kw | E617pwHkquBHUPAqcThYYmw1Wbhcwy1vq8V4vnPpAfMY | 1.03802136 SOL
7vq2GPfJhEzvxzyHCmWQAxiBvvjmFY8L36HV1MGeTv9W | E617pwHkquBHUPAqcThYYmw1Wbhcwy1vq8V4vnPpAfMY | 1.03802136 SOL
DPVboTdank8b42AFBu74SHbH8uFirWBv4Y9tLDLGPV7V | E617pwHkquBHUPAqcThYYmw1Wbhcwy1vq8V4vnPpAfMY | 1.03802136 SOL
```

### Close program
```sh
solana program close <address>
```

## Watch
https://explorer.solana.com/?cluster=custom

```sh
solana logs
```

## Common problems
### Solana program deploy
#### Account data too small for instruction
```plain
Error: Deploying program failed: RPC response error -32002: Transaction simulation failed: Error processing Instruction 0: account data too small for instruction [3 log messages]
```
You'll need to extend your program size.
```sh
solana program extend FTa31aKNoLQJTW3C2XazWemfnWWbHckaRzaYzLR25Wio 20000 -u l -k ${HOME}/.config/solana/id.json 
```
`FTa31aKN...` PROGRAM_ID

`20000` ADDITIONAL_BYTES

```plain
-u, --url <URL_OR_MONIKER>             URL for Solana's JSON RPC or moniker (or their first letter): [mainnet-beta,
                                           testnet, devnet, localhost]
-k, --keypair <KEYPAIR>                Filepath or URL to a keypair
```