// YDOTOOL
use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Waiting 2 seconds before pasting...");
    println!("Switch to another window to see the paste action!");
    
    // Give you time to switch to another window
    thread::sleep(Duration::from_secs(2));
    
    println!("Simulating Ctrl+V now...");
    
    // Use ydotool for Wayland
    Command::new("ydotool")
        .env("YDOTOOL_SOCKET", "/tmp/.ydotool_socket")
        .arg("key")
        .arg("29:1")  // Ctrl down
        .arg("47:1")  // v down
        .arg("47:0")  // v up
        .arg("29:0")  // Ctrl up
        .status()
        .expect("Failed to execute ydotool");
    
    println!("Done!");
}