// System Crates
use std::{
    sync::{
        Arc, 
        Mutex, 
        atomic::{ 
            AtomicBool, 
            Ordering 
        }
    }, 
    thread::sleep, 
    time::Duration
};

// External Crates
use arboard::Clipboard;

// My Crates
use crate::{
    common::{ 
        ClipboardItem, 
        GetItem 
    },
    history::ClipboardHistory
};

pub fn poll(stop_signal:Arc<AtomicBool>, history: Arc<Mutex<ClipboardHistory>>) -> () {
    // Create a new clipboard service
    let mut clipboard_service = Clipboard::new().unwrap();

    let empty_item = ClipboardItem::Text("".to_string());

    // Get the current item in clipboard. This will be compared with and edited
    let mut last_item = match clipboard_service.get_item() {
        Ok(item) => {item},
        Err(_) => {empty_item.clone()},
    };

    while !stop_signal.load(Ordering::SeqCst) {
        // Poll every 500ms
        sleep(Duration::from_millis(500));

        // Item Checking
        let current_item = match clipboard_service.get_item() {
            Ok(item) => {item},
            Err(_) => {empty_item.clone()},
        };

        // Only new item
        if current_item != last_item {

            // Acquire Lock
            match history.try_lock() {
                Ok(mut unlocked_history) => {
                    // Add item to history
                    unlocked_history.add(current_item.clone());

                    // Update last item
                    last_item = current_item;
                },
                Err(_) => {/* Failed To Get Lock, Skip */},
            }
            
        }
    }
}