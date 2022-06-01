# Bitcoin-Cosign

A collaborative custody service allowing users to protect their bitcoins by locking them to a 2-of-3 multisignature address. The address is created from two (2) user-supplied extended public keys and one provided by the service. This allows users to unlock their bitcoins independently by two valid signatures, or collaboratively, by providing one valid signature, and getting the second from the service.

## MVP Status
- Work in progess. 

## Features
This project is built with `Rust` and `actix-web` as a collection of API endpoints to:
- [x] Register new users.
- [x] Collect and persist extended public keys from users.
- [x] Generate multisignature destination addresses where users' bitcoins can be locked to
- [ ] Collaborate in the signing of transaction inputs referencing the UTXOs locked to 2-of-3 multisignature address. 

## Required Dependencies
- Rust v1.56+

## Run
1. Clone the repository and change to the cloned directory
```sh
$ git clone https://github.com/enigbe/bitcoin-cosign
$ cd bitcoin-cosign
```
2. Start a docker container for the database
```sh
$ ./scripts/init_db.sh
```
3. Run unit and integration tests. Ensure all tests are passing before moving to the next step
```sh
$ cargo test
```
4. Start the server
```sh
$ cargo run
```
You should have the server running on port `33335` if all goes well.