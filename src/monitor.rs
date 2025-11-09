// Standard Crates
use std::{
    thread::{
        self,
        sleep
    }, 
    time::Duration, 
    io::{
        stdin, 
        stdout, 
        Write
    },
    sync::{
        Arc, 
        Mutex,
        atomic::{
            AtomicBool, 
            Ordering
        }
    }
};

// External Crates
use arboard::{Clipboard};
use termion::{
    event::Key, 
    input::TermRead, 
    raw::IntoRawMode
};

// Custom Crates
use crate::common::GetItem;

// -------------------- Monitor, just for fun ------------------------
#[allow(unused)]
pub trait Monitor {
    fn monitor(self);
}

impl Monitor for Clipboard {
    // Usage:
    // let clipboard = Clipboard::new().unwrap();
    // clipboard.monitor();
    
    fn monitor(self) {        
        let stop = Arc::new(AtomicBool::new(false));

        let kb_stop = stop.clone();
        let cm_stop = stop.clone();
        

        let kb_handle = thread::spawn(move || {
            let stdin = stdin();
            let mut stdout = stdout().into_raw_mode().unwrap();

            write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
            stdout.flush().unwrap();

            write!(stdout, "Monitoring Clipboard. Press 'q' to exit. \r\n").unwrap();
            stdout.flush().unwrap();

            for c in stdin.keys() {
                if kb_stop.load(Ordering::SeqCst) {
                    break;
                }

                if let Ok(Key::Char('q')) | Ok(Key::Char('Q')) = c {
                    kb_stop.store(true, Ordering::SeqCst);
                    break;
                }
            }
        });
        
        let clipboard = Arc::new(Mutex::new(self));
        
        let cm_handle = thread::spawn(move || {
            let mut stdout = stdout().into_raw_mode().unwrap();

            let mut previous_content = clipboard.lock().unwrap().get_item().unwrap();
            
            while !cm_stop.load(Ordering::SeqCst) {
                sleep(Duration::from_millis(100));

                if let Ok(content) = clipboard.lock().unwrap().get_item() {
                    if content != previous_content {
                        write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
                        
                        write!(stdout, "Monitoring Clipboard. Press 'q' to exit. \r\n").unwrap();
                        stdout.flush().unwrap();

                        write!(stdout, "\n\nClipboard Change Detected:\r\n\n```\r\n{}\r\n```\r\n", content).unwrap();
                        stdout.flush().unwrap();
                        
                        previous_content = content;
                    }
                }
            }
        });

        kb_handle.join().unwrap();
        cm_handle.join().unwrap();

    }
}
// -------------------------------------------------------------------