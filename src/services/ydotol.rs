// YDOTOOL
use std::process::Command;
use std::path::Path;

pub fn send_shift_insert() {
    // Check if socket exists
    let socket_path = "/tmp/.ydotool_socket";
    if !Path::new(socket_path).exists() {
        eprintln!("ydotool socket not found at {}", socket_path);
        return;
    }

    // Simulate Shift+Insert (paste)
    let result = Command::new("ydotool")
        .env("YDOTOOL_SOCKET", socket_path)
        .args([
            "key",
            "42:1", // Shift down
            "110:1", // Insert down
            "110:0", // Insert up
            "42:0", // Shift up
        ])
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("ydotool failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => eprintln!("Failed to execute ydotool: {}", e),
    }
}