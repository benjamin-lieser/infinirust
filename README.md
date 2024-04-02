# Infinirust
A multiplayer voxel game implemented in Rust
## Features
- Procedially generated voxel world
- Multiple Players can join a server
- Block Update are propagated to the players (TODO)
- Player positions are propagated (TODO)
## Technical Features
- Async TCP Server
- Multithreaded OpenGL Client, to prevent lag spikes
## How to play locally
Start the game with `cargo run --release --bin client world_dir PlayerName`
This will compile and start the internal server and logs in with name PlayerName
## How to play remotly
Start the server with `cargo run --release --bin server ip:port world_dir`
Connect a client with `not yet implemented`