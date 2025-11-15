// Standard Crates
use std::{fs, process, thread, time::Duration};

// External Crates
use clap::{Parser, Subcommand};

// My Crates
use super_v::{
    common::{LOCK_PATH, SOCKET_PATH},
    gui::clipboard_gui::{InitialTab, MainThreadMsg, run_gui},
    services::{clipboard_manager::Manager, ydotool::send_shift_insert},
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

    /// Opens the emoji screen
    OpenEmoji,

    /// Cleans any leftovers
    Clean,
}

#[derive(Parser, Debug)]
#[command(
    name = "super_v",
    version = "1.0.0",
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
        }
        Err(_) => {
            eprintln!("Another instance of Manager already running.");
            process::exit(0);
        }
    };

    c_manager.start_daemon();
}

// ----------------------------- Main --------------------------------
fn main() {
    // Daemon
    let args = Args::parse();
    match args.command {
        Command::Start => {
            start_manager_daemon();
        }
        Command::OpenGui => {
            use std::sync::mpsc::channel;

            // Create a simple streaming channel
            let (tx, rx) = channel::<MainThreadMsg>();

            let ydotool_handle = std::thread::spawn(move || {
                while let Ok(msg) = rx.recv() {
                    match msg {
                        MainThreadMsg::AutoPaste => {
                            thread::sleep(Duration::from_millis(100));
                            send_shift_insert();
                        }
                        MainThreadMsg::Close => {
                            break;
                        }
                    }
                }
            });

            // Should be in main thread
            run_gui(tx, InitialTab::Clipboard);
            let _ = ydotool_handle.join();
        }
        Command::OpenEmoji => {
            use std::sync::mpsc::channel;

            // Create a simple streaming channel
            let (tx, rx) = channel::<MainThreadMsg>();

            let ydotool_handle = std::thread::spawn(move || {
                while let Ok(msg) = rx.recv() {
                    match msg {
                        MainThreadMsg::AutoPaste => {
                            thread::sleep(Duration::from_millis(100));
                            send_shift_insert();
                        }
                        MainThreadMsg::Close => {
                            break;
                        }
                    }
                }
            });

            // Should be in main thread
            run_gui(tx, InitialTab::Emoji);
            let _ = ydotool_handle.join();
        }
        Command::Clean => {
            let _ = fs::remove_file(SOCKET_PATH);
            let _ = fs::remove_file(LOCK_PATH);
        }
    }
}
// -------------------------------------------------------------------
