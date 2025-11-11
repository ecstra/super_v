// Standard Crates
#[allow(unused)]
use std::{
    fmt,
    error::Error
};

// External Crates
use arboard::Clipboard;
use serde::{
    Serialize, 
    Deserialize
};

// ------------------------- Constants --------------------------------
pub const SOCKET_PATH: &str = "/tmp/super_v.sock";
pub const LOCK_PATH: &str = "/tmp/super_v.lock";
// --------------------------------------------------------------------


// --------------------------- Errors --------------------------------
/// Error types for clipboard operations.
#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum ClipboardError {
    /// Returned when attempting to access an empty clipboard
    ClipboardEmpty,

    /// Returned when trying to access an item out of bounds.
    IndexOutOfBound
}

/// Error Type for Clipboard Manager Daemon
#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum DaemonError {
    /// Returned when attempting to spawn Manager but an instance is already running.
    ManagerMultiSpawn,

    IPCErr(IPCServerError)
}

/// Error Type for IPCServer
#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum IPCServerError {
    FileNotFound,
    ConnectionError(String),
    BindError(String),
    SendError(String),
}

// Displays for the Errors
impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardError::ClipboardEmpty => {
                write!(f, "Clipboard is empty. Please add copy something before trying again.")
            },
            ClipboardError::IndexOutOfBound => {
                write!(f, "Item position not found or out-of-bounds.")
            }
        }
    }
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonError::ManagerMultiSpawn => {
                write!(f, "An instance of the Manager is already open.")
            },
            DaemonError::IPCErr(ipc_error) => {
                write!(f, "IPC Error: {}", ipc_error)
            }
        }
    }
}

impl fmt::Display for IPCServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IPCServerError::ConnectionError(string) => {
                write!(f, "Could not connect to socket: {}", string)
            },
            IPCServerError::BindError(string) => {
                write!(f, "Could not bind to socket: {}", string)
            },
            IPCServerError::SendError(string) => {
                write!(f, "Could not send item: {}", string)
            }
            IPCServerError::FileNotFound => {
                write!(f, "Sock file is missing?")
            }
        }
    }
}


// Implement the structs as Errors
impl Error for ClipboardError {}
impl Error for DaemonError {}
impl Error for IPCServerError {}
// -------------------------------------------------------------------


// ----------------------- Clipboard Item ----------------------------
/// Represents an item that can be stored in the clipboard.
/// 
/// This enum supports both text and image data types, allowing the clipboard
/// to handle multiple content formats.
#[allow(unused)]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ClipboardItem {
    /// Plain text content
    Text(String),
    
    /// Image content with dimensions and raw bytes
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

/// Trait for retrieving clipboard content as a ClipboardItem.
/// 
/// This trait provides a unified interface for getting clipboard content,
/// automatically detecting whether the content is text or an image.
#[allow(unused)]
pub trait GetItem {
    /// Retrieves the current clipboard content.
    /// 
    /// # Returns
    /// 
    /// * `Ok(ClipboardItem)` - The clipboard content as either Text or Image
    /// * `Err(ClipboardErr::ClipboardEmpty)` - If the clipboard is empty
    fn get_item(&mut self) -> Result<ClipboardItem, ClipboardError>;
}

impl GetItem for Clipboard {
    /// Implementation of GetItem for arboard's Clipboard.
    /// 
    /// Attempts to retrieve clipboard content in the following order:
    /// 1. Image data (if available)
    /// 2. Text data (if available)
    /// 3. Returns ClipboardEmpty error if neither is available
    fn get_item(&mut self) -> Result<ClipboardItem, ClipboardError> {
        if let Ok(img_dat) = self.get_image() {
            Ok(ClipboardItem::Image { 
                width: img_dat.width, 
                height: img_dat.height, 
                bytes: img_dat.bytes.to_vec()
            })
        } else if let Ok(str_data) = self.get_text() {
            Ok(ClipboardItem::Text(str_data))
        } else {
            Err(ClipboardError::ClipboardEmpty)
        }
    }
}
// -------------------------------------------------------------------