// Standard Crates
#[allow(unused)]
use std::{
    collections::{HashMap, VecDeque}, error::Error, fmt, thread::sleep, time::Duration
};

// External Crates
use arboard::Clipboard;


// // ---------------------------- Error --------------------------------
// #[derive(Debug)]
// #[allow(unused)]
// // Error when you try to overwrite a Pos
// enum PosError {
//     PosOverlap {
//         pos: u8,
//         existing_content: String,
//         new_content: String
//     },
//     PosJump {
//         prev_pos: u8,
//         pos: u8
//     }
// }

// // Displays for the Errors
// impl fmt::Display for PosError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             PosError::PosOverlap {pos, existing_content, new_content} => {
//                 write!(f, "POS ALREADY EXISTS! POS: {} | EXISTING CONTENT: {} | NEW CONTENT: {}", pos, existing_content, new_content)
//             },
//             PosError::PosJump {prev_pos, pos} => {
//                 write!(f, "WARNING! POS JUMP FOUND! PREVIOUS POS: {} | CURRENT POS: {}", prev_pos, pos)
//             }
//         }
//     }
// }

// // Implement the structs as Errors
// impl Error for PosError {}
// // -------------------------------------------------------------------


// ---------------------- Clipboard Structs --------------------------
// Note: Using "C" for now.
// Consider using specific types from arboard later on.
#[allow(unused)]
#[derive(Debug, Clone, PartialEq)] // PartialEQ needed for comparision
enum ClipboardItem {
    Text(String)
}

impl fmt::Display for ClipboardItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardItem::Text(s) => write!(f, "{}", s),
        }
    }
}

#[allow(unused)]
struct ClipboardHistory {
    history: VecDeque<ClipboardItem>,
    max_size: usize,
}
// -------------------------------------------------------------------


// --------------------- Hist Implementation -------------------------
#[allow(unused)]
impl ClipboardHistory {
    fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    // Adds a new Clipboard item to history
    fn add(&mut self, item: ClipboardItem) {
        // Check for item duplicates
        if let Some(pos) = self.history.iter().position(|i| i == &item) {
            // It already exists. Promote it.
            self.promote(pos);
            return;
        }

        // Add to 0 (front)
        self.history.push_front(item);

        // Remove old items as size exceeds
        if self.history.len() > self.max_size {
            self.history.pop_back();
        }
    }

    // Given an index, it will push it to the TOP
    fn promote(&mut self, pos: usize) {
        // Remove item as 'pos'th index
        let promoted_item = self.history.remove(pos).unwrap();

        // Add it to history's TOP
        self.history.push_front(promoted_item);
    }

    // Returns all Items of the clipboard history
    fn get_items(&self) -> &VecDeque<ClipboardItem> {
        &self.history
    }

}

// Display for ClipboardHistory is now much simpler
impl fmt::Display for ClipboardHistory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut printable = String::from("POS     | ITEM     ");
        printable += "\r\n---------------";
        
        // No sorting needed! Just iterate.
        for (pos, item) in self.history.iter().enumerate() {
            printable += &format!("\r\n{}       | {}     ", pos, item);
        }
        
        write!(f, "{printable}")
    }
}
// -------------------------------------------------------------------


// -------------------- Monitor, just for fun ------------------------
#[allow(unused)]
trait Monitor {
    fn monitor(&mut self);
}

impl Monitor for Clipboard {
    // Usage:
    // let mut clipboard = Clipboard::new().unwrap();
    // clipboard.monitor();
    
    fn monitor(&mut self) {
        println!("Starting to monitor. Press 'CTRL + C' in terminal to quit.");

        let mut previous_content = self.get_text().unwrap();
        loop {
            let content = self.get_text().unwrap();
            
            if content != previous_content {
                println!("<--------------------->");
                println!("Clipboard Change Detected: {}", self.get_text().unwrap());
                previous_content = content;
            }

            // To prevent overloading? 
            // (perhaps not an issue in rust like it is on python)
            sleep(Duration::from_millis(100));
        }
    }
}
// -------------------------------------------------------------------


// ----------------------------- Main --------------------------------
fn main() {
    let mut ch = ClipboardHistory::new(20);
    ch.add(ClipboardItem::Text("miaow".to_string()));
    ch.add(ClipboardItem::Text("woof".to_string()));
    ch.add(ClipboardItem::Text("rawr".to_string()));
    ch.promote(1);
    println!("{ch}");
}
// -------------------------------------------------------------------