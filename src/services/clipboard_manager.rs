// System Crates
use std::{
    fs::{
        File,
        OpenOptions, 
        remove_file
    }, 
    io::Write, 
    os::unix::net::{
        UnixListener, 
        UnixStream
    }, 
    sync::{
        Arc, 
        Mutex, 
        atomic::{ 
            AtomicBool, 
            Ordering 
        }
    }, thread::{
        self, 
        JoinHandle, 
        sleep
    }, 
    time::Duration
};

// External Crates
use arboard::Clipboard;
use fs2::FileExt;

// My Crates
use crate::{
    common::{ 
        ClipboardItem, 
        GetItem,
        DaemonError,
        LOCK_PATH,
        SOCKET_PATH
    },
    services::clipboard_ipc_server::{
        create_bind,
        read_payload,
        send_payload,
        CmdIPC,
        IPCResponse,
        Payload
    },
    history::ClipboardHistory
};

/// # Manager
///  Holds shared services and thread handles for the clipboard manager.
///
/// Fields:
/// - _clipboard_service: Arc-wrapped clipboard service used to read the system clipboard.
/// - _shared_history: Arc-wrapped ClipboardHistory shared between threads.
/// - _stop_signal: Atomic flag used to request worker threads to stop.
/// - _polling_handle: Optional JoinHandle for the polling thread.
/// - _command_handle: Optional JoinHandle for the command-handling thread.
///
/// These fields are internal to the implementation and not intended for public API use.
/// Check implementation of Manager for usage.
pub struct Manager {
    // Needed for operation
    pub _clipboard_service: Arc<Mutex<Clipboard>>,
    pub _shared_history: Arc<Mutex<ClipboardHistory>>,
    pub _stop_signal:Arc<AtomicBool>,

    // Thread handles
    pub _polling_handle: Option<JoinHandle<()>>,
    pub _command_handle: Option<JoinHandle<()>>,

    // Lock file to prevent multiple starts.
    pub _lock_file: Option<File>,

    // IPC
    pub _server: UnixListener
}

impl Manager {
    // Clipboard Size
    const CLIPBOARD_SIZE: usize = 25;

    /// Create a new Manager instance and configure global handlers.
    ///
    /// **Behavior**:
    /// - Allocates a ClipboardHistory with a fixed capacity.
    /// - Creates and wraps a Clipboard service in an Arc<Mutex<...>>.
    /// - Creates an Arc<AtomicBool> stop signal used by worker threads.
    /// - Installs a ctrl-c handler that updates the stop signal.
    /// - Has a process lock so duplicate processes can't be run.
    ///
    /// **Panics / errors**:
    /// - This constructor unwraps the clipboard creation and will panic if the clipboard cannot be initialized.
    ///
    /// **Returns**:
    /// - A fully constructed Manager with no active thread handles.
    pub fn new() -> Result<Self, DaemonError> {
        // New history
        let history: Arc<Mutex<ClipboardHistory>> = Arc::new(
            Mutex::new(
                ClipboardHistory::new(Self::CLIPBOARD_SIZE)
            )
        );

        // Clipboard service
        let _clipboard_service: Arc<Mutex<Clipboard>> = Arc::new(
            Mutex::new(
                match Clipboard::new() {
                    Ok(clipboard) => {clipboard},
                    Err(err) => {
                        panic!("ERROR: {:?}", err);
                    }
                }
            )
        );

        // Stop signal
        let _stop_signal: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        
        // Setup ctrl+c
        let ss_clone = _stop_signal.clone();
        let _ = ctrlc::set_handler(move || {
            // When ctrl+c is detected, set true
            ss_clone.store(true, Ordering::SeqCst);
        });

        // Try lock
        let lock_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(LOCK_PATH)
            .expect("Failed to open lock file");

        // Return error if lock fails
        if let Err(_) = lock_file.try_lock_exclusive() {
            return Err(DaemonError::ManagerMultiSpawn);
        }

        // Write pid for reference
        let _ = lock_file.set_len(0);
        let _ = write!(&lock_file, "{}", std::process::id());
        let _ = lock_file.sync_all();
        
        // Once file lock is gotten, create a new IPC Server
        // But first clear the previous sock file. Since we know we are the main owner of the manager.
        let _ = remove_file(SOCKET_PATH);
        let server = match create_bind().map_err(|err| DaemonError::IPCErr(err)) {
            Ok(server) => {server},
            Err(err) => {
                return Err(err);
            }
        };

        // Return the manager object
        Ok(Self {
            _clipboard_service: _clipboard_service,
            _shared_history: history,
            _stop_signal: _stop_signal,

            // No handles yet.
            _polling_handle: None,
            _command_handle: None,

            // New Listener
            _lock_file: Some(lock_file),

            // Ipc Server
            _server: server
        })
    }

    /// Start the polling service in a new background thread.
    ///
    /// **Behavior**:
    /// - Returns early with a log if a polling thread is already running.
    /// - Clones required Arcs for use inside the spawned thread.
    /// - The thread repeatedly:
    ///     * Sleeps for a fixed interval (500 ms).
    ///     * Attempts to read the current clipboard item (falling back to an empty item on error).
    ///     * Compares it with the last seen item and, if different, attempts to push it into ClipboardHistory.
    /// - Uses try_lock on locks to avoid blocking other threads; if a lock is unavailable it skips that iteration.
    /// - Exits when the stop signal is set.
    ///
    /// **Notes**:
    /// - This function stores the JoinHandle in _polling_handle.
    /// - Designed to be safe for use alongside a command thread that may also access shared_history.
    pub fn _polling_service(&mut self) {
        // Check if polling thread is already started
        let None = self._polling_handle else {
            println!("Polling service is already running");
            return;
        };

        // Create clones of the Arc items needed.
        let clipboard_service = self._clipboard_service.clone();
        let stop_signal = self._stop_signal.clone();
        let shared_history = self._shared_history.clone();
        println!("Started Polling...");

        // Start the polling in a thread and store the handle
        self._polling_handle = Some(thread::spawn(move || {
            println!("Started Polling in thread...");

            let empty_item = ClipboardItem::Text("".to_string());

            // Get the current item in clipboard. This will be compared with and edited
            let mut last_item = match clipboard_service.try_lock() {
                Ok(mut unlocked_clipboard) => {
                    match unlocked_clipboard.get_item() {
                        Ok(item) => {item},
                        Err(_) => {empty_item.clone()},
                    }
                },
                Err(_) => {empty_item.clone()},
            };
            
            while !stop_signal.load(Ordering::SeqCst) {
                // Item Checking
                let current_item = match clipboard_service.try_lock() {
                    Ok(mut unlocked_clipboard) => {
                        match unlocked_clipboard.get_item() {
                            Ok(item) => {item},
                            Err(_) => {empty_item.clone()},
                        }
                    },
                    Err(_) => {empty_item.clone()},
                };

                // This should be fine since _polling_service and _command_service both exist in the same process.
                // So no need for thread-to-thread communication management and can purely focus on IPC management.
                // Checks if item is new or not.
                if current_item != last_item {
                    // Acquire Lock
                    match shared_history.try_lock() {
                        Ok(mut unlocked_history) => {
                            // Add item to history
                            unlocked_history.add(current_item.clone());
                            println!("Curent History: \n{}\r\n", unlocked_history);

                            // Update Last item
                            last_item = current_item
                        },
                        Err(_) => {/* Failed To Get Lock, Skip */},
                    }
                }

                // Poll every 100ms
                sleep(Duration::from_millis(100));
            }
        }));
    }

    /// Start the command-handling service in a background thread.
    ///
    /// **Behavior**:
    /// - Listens for incoming IPC messages from external processes.
    /// - Parses commands serialized as CmdIPC variants (e.g., Promote, Delete, Snapshot, Clear).
    /// - Executes the requested operation on the shared ClipboardHistory instance.
    /// - Constructs an IPCResponse containing:
    ///     - A current snapshot of the ClipboardHistory.
    ///     - An optional message describing the operation result.
    /// - Sends the serialized IPCResponse back through IPC to the caller.
    ///
    /// **Notes**:
    /// - This service runs concurrently and in the same process with the clipboard polling thread (or it won't work).
    /// - Should store the thread JoinHandle in _command_handle.
    pub fn _command_service(&mut self) {

        // Clone the items needed.
        let stop_signal = self._stop_signal.clone();
        let shared_history: Arc<Mutex<ClipboardHistory>> = self._shared_history.clone();

        // Find another way to just own the server instead of cloning.
        let ipc_server = self._server.try_clone().unwrap();

        // Helper functions to send snapshot and err
        fn _send_snapshot(s: &mut UnixStream, snapshot: ClipboardHistory) {
            send_payload(s, Payload::Resp(IPCResponse {
                history_snapshot: Some(snapshot),
                message: None
            }));
        }

        fn _send_err(s: &mut UnixStream, msg: &str) {
            send_payload(s, Payload::Resp(IPCResponse {
                history_snapshot: None,
                message: Some(msg.to_string())
            }));
        }

        // Run the command service in a new thread
        // The thread will consume the only UnixListener (since it's not an Arc) which is fine
        // Then it will listen for streams which send CmdIpc as Payload
        // Parse the Cmd and apply operation on the clipboard history
        // Finally, send a snapshot of the history
        self._command_handle = Some(thread::spawn(move || {

            println!("Listening for Commands");

            // Handle incoming messages
            for stream in ipc_server.incoming() {
                // Break the loop if stop_signal is found
                if stop_signal.load(Ordering::SeqCst) {break}

                match stream {
                    Ok(mut s ) => {
                        let history_for_thread = shared_history.clone();

                        // Handle payload in another thread
                        thread::spawn(move || {
                            // Read the payload
                            let payload = read_payload(&mut s);
                            
                            println!("Recieved Payload: {:?}", payload);

                            // Match the payload and execute command
                            match payload {
                                Payload::Cmd(ipc_cmd) => {
                                    match ipc_cmd {
                                        CmdIPC::Clear => {
                                           // Get mutex guard
                                            match history_for_thread.lock() {
                                                Ok(mut unlocked_history) => {
                                                    // Clear the history
                                                    unlocked_history.clear();
                                                    
                                                    // Create snapshot, drop guard, send snapshot
                                                    let snapshot = unlocked_history.clone();
                                                    _send_snapshot(&mut s, snapshot);
                                                },
                                                Err(_) => {
                                                    _send_err(&mut s, "Could not unlock history");
                                                }
                                            }
                                        },
                                        CmdIPC::Delete(pos) => {
                                           // Get mutex guard
                                            match history_for_thread.lock() {
                                                Ok(mut unlocked_history) => {
                                                    // Delete the item 
                                                    match unlocked_history.delete(pos) {
                                                        Ok(_) => {
                                                            // Create snapshot, drop guard, send snapshot
                                                            let snapshot = unlocked_history.clone();
                                                            _send_snapshot(&mut s, snapshot);
                                                        },
                                                        Err(_) => {_send_err(&mut s, "Could not delete item. Index out of bounds.");},
                                                    };
                                                    
                                                    
                                                },
                                                Err(_) => {
                                                    _send_err(&mut s, "Could not unlock history");
                                                }
                                            }
                                        },
                                        CmdIPC::Promote(pos) => {
                                            // Get mutex guard
                                            match history_for_thread.lock() {
                                                Ok(mut unlocked_history) => {
                                                    // Promote the item 
                                                    match unlocked_history.promote(pos) {
                                                        Ok(_) => {
                                                            // Create snapshot, drop guard, send snapshot
                                                            let snapshot = unlocked_history.clone();
                                                            _send_snapshot(&mut s, snapshot);
                                                        },
                                                        Err(_) => {_send_err(&mut s, "Could not promote item. Index out of bounds.");},
                                                    };
                                                },
                                                Err(_) => {
                                                    _send_err(&mut s, "Could not unlock history");
                                                }
                                            }
                                        },
                                        CmdIPC::Snapshot => {
                                            // Get mutex guard
                                            match history_for_thread.lock() {
                                                Ok(unlocked_history) => {
                                                    // Create snapshot, drop guard, send snapshot
                                                    let snapshot = unlocked_history.clone();
                                                    _send_snapshot(&mut s, snapshot);
                                                },
                                                Err(_) => {
                                                    // Send err if could not unlock
                                                    _send_err(&mut s, "Could not unlock history");
                                                }
                                            }
                                        }
                                    }
                                },
                                Payload::Resp(_) => {
                                    _send_err(&mut s, "Wrong Payload type recieved. Expected CmdIpc but got IPCResponse.");
                                }
                            }
                        });
                    },
                    Err(e) => {
                        eprintln!("Accept Error: {e}");
                    },
                }
            }
            
        }));
    }
    
    /// Start all configured background services.
    ///
    /// **Behavior**:
    /// - Calls _polling_service to start the clipboard poller.
    /// - Calls _command_service to start command handling.
    /// - Each service checks whether it is already running and will not start duplicate
    pub fn start_daemon(&mut self) {
        // Start the polling service
        self._polling_service();

        // Start the command service
        self._command_service();

        // Clone a stop signal
        let daemon_stop_signal = self._stop_signal.clone();

        // Block until ctrl-c or other code sets the stop flag
        while !daemon_stop_signal.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
        }

        println!("Stop signal recieved. Stopping!");

        // Shutdown when daemon stops
        self.stop();
    }

    /// Request shutdown and join worker threads.
    ///
    /// **Behavior**:
    /// - Sets the stop signal to request all worker threads to exit.
    /// - Takes ownership of the stored thread handles and attempts to join them.
    /// - Joining is performed from a short-lived helper thread to avoid blocking the caller.
    ///
    /// **Notes**:
    /// - After stop returns, worker threads will have been requested to stop and any existing handles will be joined.
    /// - This method swallows join errors and does not return a failure result.
    pub fn stop(&mut self) {
        // Signal threads to stop
        self._stop_signal.store(true, Ordering::SeqCst);

        // Take the handles
        let _polling_handle = self._polling_handle.take();
        let _command_handle = self._command_handle.take();

        // Spawn a short-lived thread to join them so main thread is not blocked
        // All errors are swallowed
        let _ = thread::spawn(move || {
            if let Some(h) = _polling_handle {
                let _ = h.join();
            }
            if let Some(h) = _command_handle {
                let _ = h.join();
            }
        });

        // Unlock the lock file
        // Swallows the error.
        if let Some(lockfile) = &self._lock_file {
            let _ = lockfile.unlock();
            let _ = remove_file(SOCKET_PATH);
            let _ = remove_file(LOCK_PATH);
        }
    }
}