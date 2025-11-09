// Standard Crates
#[allow(unused)]
use std::{
    fmt,
    error::Error
};

// External Crates
use arboard::{Clipboard};

// ---------------------------- Error --------------------------------
#[derive(Debug)]
#[allow(unused)]
// Error when you try to overwrite a Pos
pub enum ClipboardErr {
    ClipboardEmpty
}

// Displays for the Errors
impl fmt::Display for ClipboardErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardErr::ClipboardEmpty => {
                write!(f, "Clipboard is empty. Please add copy something before trying again.")
            }
        }
    }
}

// Implement the structs as Errors
impl Error for ClipboardErr {}
// -------------------------------------------------------------------


// ------------------------- Clipboard Item -----------------------------
#[allow(unused)]
#[derive(Debug, Clone, PartialEq)] // Debuggable, Cloneable and Comparable.
pub enum ClipboardItem {
    Text(String),
    Image {
        width: usize,
        height: usize,
        bytes: Vec<u8>
    }
}

// Make the item printable
impl fmt::Display for ClipboardItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardItem::Text(s) => write!(f, "{}", s.replace('\n', "\r\n")),
            ClipboardItem::Image {width, height, ..} => write!(f, "[Image: {width}x{height}]")
        }
    }
}

// Compatibility with arboard
#[allow(unused)]
pub trait GetItem {
    fn get_item(&mut self) -> Result<ClipboardItem, ClipboardErr>;
}

// Implementation for arboard
impl GetItem for Clipboard {
    fn get_item(&mut self) -> Result<ClipboardItem, ClipboardErr> {
        if let Ok(img_dat) = self.get_image() {
            Ok(ClipboardItem::Image { 
                width: img_dat.width, 
                height: img_dat.height, 
                bytes: img_dat.bytes.to_vec()
            })
        } else if let Ok(str_data) = self.get_text() {
            Ok(ClipboardItem::Text(str_data))
        } else {
            Err(ClipboardErr::ClipboardEmpty)
        }
    }
}
// -------------------------------------------------------------------