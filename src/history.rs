use core::panic;
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
/// A clipboard history manager that maintains a fixed-size queue of clipboard items.
/// 
/// This structure keeps track of clipboard items in a VecDeque, automatically managing
/// the history size and handling duplicate items by promoting them to the top.
#[allow(unused)]
pub struct ClipboardHistory {
    history: VecDeque<ClipboardItem>,
    max_size: usize,
}

#[allow(unused)]
impl ClipboardHistory {
    /// Creates a new ClipboardHistory with the specified maximum size.
    /// 
    /// # Arguments
    /// 
    /// * `max_size` - The maximum number of items to keep in history
    pub fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Adds a new clipboard item to the history.
    /// 
    /// If the item already exists in history, it will be promoted to the front
    /// instead of creating a duplicate. If the history exceeds max_size after
    /// adding, the oldest item is removed.
    /// 
    /// # Arguments
    /// 
    /// * `item` - The ClipboardItem to add to history
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

    /// Promotes an item at the given position to the front of the history.
    /// 
    /// # Arguments
    /// 
    /// * `pos` - The index of the item to promote
    /// 
    /// # Panics
    /// 
    /// Panics if the position is out of bounds
    pub fn promote(&mut self, pos: usize) {
        // Remove item as 'pos'th index
        let promoted_item = match self.history.remove(pos) {
            Some(item) => {
                self.history.push_front(item)
            },
            None => {
                // No item found
                panic!("Item position not found or out-of-bounds.");
            },
        };
    }

    /// Returns a reference to all items in the clipboard history.
    /// 
    /// Items are ordered from most recent (front) to oldest (back).
    pub fn get_items(&self) -> &VecDeque<ClipboardItem> {
        &self.history
    }

    /// Clears all items from the clipboard history.
    pub fn clear(&mut self) {
        self.history.clear();
    }

}

impl fmt::Display for ClipboardHistory {
    // Display for ClipboardHistory is now much simpler
    /// Formats the clipboard history as a human-readable table.
    /// 
    /// Displays each item with its position and content. Text items show their
    /// content, while image items show their dimensions.
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