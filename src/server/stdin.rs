use std::io::{BufRead, Write};

use crate::server::NOUSER;

use super::ServerCommand;

/// Supposed to be started in its own thread handling sdtin in a blocking way
pub fn handle_stdin(server: ServerCommand, bind: String) {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let command = line.unwrap_or_else(|e| {
            _ = server.blocking_send((NOUSER, super::Command::Shutdown));
            eprintln!("IO error in stdin: {}", e);
            panic!();
        });
        match command.as_str() {
            "exit" => {
                //If the server is already down exit the process
                server
                    .blocking_send((NOUSER, super::Command::Shutdown))
                    .unwrap_or_else(|_| std::process::exit(1));
            }
            "bind" => {
                //Writes the address to connect to on stdout
                println!("{}", bind);
                std::io::stdout().flush().unwrap();
            }
            _ => {
                eprintln!("Unknown command");
            }
        }
    }
    //Reached EOF, if the server is already down exit the process
    eprintln!("Server stdin EOF");
    server
        .blocking_send((NOUSER, super::Command::Shutdown))
        .unwrap_or_else(|_| std::process::exit(1));
}
