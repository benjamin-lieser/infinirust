use std::io::BufRead;

use super::ServerCommand;


/// Supposed to be started in its own thread handling sdtin in a blocking way
pub fn handle_stdin(server: ServerCommand) {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let command = line.unwrap_or_else(|e| {
            _ = server.blocking_send(super::Command::Shutdown);
            eprintln!("IO error in stdin: {}", e);
            panic!();
        });
        match command.as_str() {
            "exit" => {
                _ = server.blocking_send(super::Command::Shutdown);
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
    //Reached EOF
    _ = server.blocking_send(super::Command::Shutdown);
}