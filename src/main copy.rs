// Standard Crates
use std::{
    fs::remove_file, process
};

// External Crates
use clap::{
    Parser,
    Subcommand
};

// My Crates
use super_v::{
    common::{
        LOCK_PATH, 
        SOCKET_PATH
    }, 
    gui::clipboard_gui::run_gui,
    services::{
        clipboard_manager::Manager
    }
};

/*
Notes:
- Add tests
- STOP USING UNWRAP -> Look what you did -.- Now clean the unwraps and replace with proper error handling and eprintln!()...
- Threaded Clipboard Manager that has a polling mechanism and manages history.
- Keyboard simulation using rdev (to paste when item is clicked)
- Mouse pointer monitoring to open the window at cursor location
- Emoji screen
*/

#[derive(Subcommand, Debug)]
enum Command {
    /// Start the process
    Start,

    /// Open the GUI
    OpenGui,

    /// Cleans any leftovers
    Clean
}

#[derive(Parser, Debug)]
#[command(
    name = "super_v",
    version = "0.0.1",
    about = "Clipboard Service that looks like Win11",
    long_about = None
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn start_manager_daemon() {
    let mut c_manager = match Manager::new() {
        Ok(manager) => {
            println!("Starting service...");
            manager
        },
        Err(_) => {
            eprintln!("Another instance of Manager already running.");
            process::exit(0);
        },
    };

    c_manager.start_daemon();
}

// ----------------------------- Main --------------------------------
fn main() {
    // Daemon
    let args= Args::parse();
    match args.command {
        Command::Start => {
            start_manager_daemon();
        },
        Command::OpenGui => {
            run_gui();
        },
        Command::Clean => {
            let _ = remove_file(SOCKET_PATH);
            let _ = remove_file(LOCK_PATH);
        }
    }
}
// -------------------------------------------------------------------