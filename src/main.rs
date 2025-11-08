// Standard Crates
use std::{
    fmt,
    collections::HashMap, 
    error::Error, 
    thread::sleep, 
    time::Duration
};

// External Crates
use arboard::Clipboard;


// ---------------------------- Error --------------------------------
#[derive(Debug)]
#[allow(unused)]
// Error when you try to overwrite a Pos
enum PosError {
    PosOverlap {
        pos: u8,
        existing_content: String,
        new_content: String
    },
    PosJump {
        prev_pos: u8,
        pos: u8
    }
}

// Displays for the Errors
impl fmt::Display for PosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PosError::PosOverlap {pos, existing_content, new_content} => {
                write!(f, "POS ALREADY EXISTS! POS: {} | EXISTING CONTENT: {} | NEW CONTENT: {}", pos, existing_content, new_content)
            },
            PosError::PosJump {prev_pos, pos} => {
                write!(f, "WARNING! POS JUMP FOUND! PREVIOUS POS: {} | CURRENT POS: {}", prev_pos, pos)
            }
        }
    }
}

// Implement the structs as Errors
impl Error for PosError {}
// -------------------------------------------------------------------


// ---------------------- Clipboard Structs --------------------------
// Note: Using "C" for now.
// Consider using specific types from arboard later on.

#[allow(unused)]
struct ClipBoardItem<C> {
    pos: u8,
    item: C
}

#[allow(unused)]
struct ClipboardHistory<C> {
    history: HashMap<u8, C>
    // Or should it be Vec<ClipboardItem> ????
}
// -------------------------------------------------------------------


// --------------------- Hist Implementation -------------------------
// Note: Using "C" for now.
// Consider using specific types from arboard later on.

#[allow(unused)]
impl<C> ClipboardHistory<C> 
where 
    C: Clone 
{
    // Creates as new Hashmap
    fn new() -> Self {
        Self { 
            history: HashMap::new()
        }
    }

    // Adds a new item to the HashMap
    fn append(&mut self, pos: u8, item: C) -> Result<(), PosError> 
    where 
        C: fmt::Display 
    {
        // Needed Features (todo):
        // 1. Check if the same content already exists. If yes, just promote to top.
        // 2. Check if the passed pos is valid or overwriting. If overwriting, throw error.
        // 3. Check if there is a jump in pos (0, 2, 3, ...)
        // 4. Ensure proper ascenind order (no 2, 1, 3)
        // 5. Ensure positive values (no -1, -2)
        // 6. Ensure proper start (0, 1, 2... AND not 2, 3, 4...)
        // ?7. Manage Pos within history?
        
        // Check for overlapping Pos

        if self.history.contains_key(&pos) {
            return Err(PosError::PosOverlap { 
                pos, 
                existing_content: self.history.get(&pos).unwrap().to_string(), 
                new_content: item.to_string() 
            });
        }

        self.history.insert(pos, item);

        Ok(())
    }

    // Shifts an item to top of list while re-ordering the 
    fn promote(&mut self, pos: u8) {
        // Remove the value from HashMap
        let promoted_value = self.history.remove(&pos).unwrap();
        
        // Re-Order the positions.
        // Promoted value (x) => 0. Other values till (x) => x + 1
        // Holy jesus this is expensive
        self.history = self.history
        .iter()
        .map(|(key, value)| {
            let new_key = if key < &pos { key + 1 } else { *key };
            (new_key, value.clone())
        }).collect();

        // Promote the selected value to TOP
        self.history.insert(0, promoted_value);
    }

}

// Display for ClipboardHistory
impl fmt::Display for ClipboardHistory<&str> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut printable = String::from("POS     | ITEM     ");
        printable += "\r\n---------------";
        
        // Sort the keys before iterating
        let mut positions: Vec<&u8> = self.history.keys().collect();
        positions.sort();
        
        for &pos in positions {
            let item = self.history.get(&pos).unwrap();
            printable += &format!("\r\n{}       | {}     ", pos, item);
        }
        
        write!(f, "{printable}")
    }
}
// -------------------------------------------------------------------


// --------------------- Basically a monitor -------------------------
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

            sleep(Duration::from_millis(1000));
        }
    }
}
// -------------------------------------------------------------------


// ----------------------------- Main --------------------------------
fn main() {
    let mut ch = ClipboardHistory::new();
    ch.append(0, "mioaw").unwrap();
    ch.append(1, "woof").unwrap();
    ch.append(2, "rawr").unwrap();
    ch.promote(1);
    println!("{ch}");
}
// -------------------------------------------------------------------
