// Standard Crates
#[allow(unused)]
use std::{
    fmt,
    collections::{
        VecDeque
    }
};

// External Crates
use crate::common::ClipboardItem;

// --------------------- Hist Implementation -------------------------
#[allow(unused)]
pub struct ClipboardHistory {
    history: VecDeque<ClipboardItem>,
    max_size: usize,
}

#[allow(unused)]
impl ClipboardHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    // Adds a new Clipboard item to history
    pub fn add(&mut self, item: ClipboardItem) {
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
    pub fn promote(&mut self, pos: usize) {
        // Remove item as 'pos'th index
        let promoted_item = self.history.remove(pos).unwrap();

        // Add it to history's TOP
        self.history.push_front(promoted_item);
    }

    // Returns all Items of the clipboard history
    pub fn get_items(&self) -> &VecDeque<ClipboardItem> {
        &self.history
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

}

// Display for ClipboardHistory is now much simpler
impl fmt::Display for ClipboardHistory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut printable = String::from("POS     | ITEM     ");
        printable += "\r\n---------------";
        
        // No sorting needed! Just iterate.
        for (pos, item) in self.history.iter().enumerate() {
            match item {
                ClipboardItem::Image { width, height, .. } => {
                    printable += &format!("\r\n{}       | Image ({}, {})     ", pos, width, height);
                },
                ClipboardItem::Text(string) => {
                    printable += &format!("\r\n{}       | {}     ", pos, string.to_string());
                }
            }
        }
        
        write!(f, "{printable}")
    }
}
// -------------------------------------------------------------------