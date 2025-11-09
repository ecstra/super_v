#[cfg(test)]
mod clipboard_poller_test {
    use std::{sync::{Arc, Mutex, atomic::AtomicBool}, thread, time::Duration};
    use std::sync::atomic::Ordering;
    use super_v::{history::ClipboardHistory, services::clipboard_poller::poll};



    #[test]
    fn test_poller_stops_on_signal() {
        // Create History and stop signals
        let history = Arc::new(
            Mutex::new(
                ClipboardHistory::new(10)
            )
        );
        let stop_signal = Arc::new(AtomicBool::new(false));
        
        let stop_clone = stop_signal.clone();
        let hist_clone = history.clone();

        // Create a new thread for polling
        let poller_handle = thread::spawn(move || {
            poll(stop_clone, hist_clone);
        });

        // Give time
        thread::sleep(Duration::from_millis(50));

        // Send stop signal
        stop_signal.store(true, Ordering::SeqCst);

        // Give time for the poller to check the signal and exit
        thread::sleep(Duration::from_millis(600));

        // Check if thread is closed
        assert!(poller_handle.is_finished(), "Uh-oh! Poller still running after sending the Stop Signal");
        
        // Join the thread
        let _ = poller_handle.join();

    }

    // Add more tests here...
    
}