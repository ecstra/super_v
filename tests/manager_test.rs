#[cfg(test)]
mod clipboard_manager_test {
    use arboard::{Clipboard, ImageData};
    use serial_test::serial;
    use core::panic;
    use std::{
        borrow::Cow, sync::atomic::Ordering, thread, time::Duration
    };
    use super_v::{
        common::{ClipboardItem, DaemonError}, 
        services::{clipboard_ipc_server::{CmdIPC, IPCResponse, Payload, create_default_stream, read_payload, send_payload}, clipboard_manager::Manager}
    };

    // ------------------ Helper Functions ----------------------
    fn get_hopeful_history() -> Vec<ClipboardItem> {
            let item1 = ClipboardItem::Text("item1".into());
        let item2 = ClipboardItem::Text("item2".into());
        let item3 = ClipboardItem::Text("item3".into());
        let image = ClipboardItem::Image {
            width: 50,
            height: 50,
            bytes: vec![0u8; 10000],
        };
        vec![item1, item2, item3, image]
    }

    fn beam_payload(payload: Payload) -> Payload {
        // Create manager and start services
        let mut manager = Manager::new().unwrap();
        manager._polling_service();
        manager._command_service();

        // Create a new clipboard instance and update it with items
        // Since Clipboard is synced across device, updating it here should update the manager as well
        let mut clipboard_service = Clipboard::new().unwrap();

        // Update the clipboard history to have some things...
        let _ = clipboard_service.set_image(
            ImageData {
                width: 50,
                height: 50,
                bytes: Cow::from([0u8; 10000].as_ref()),
            }
        );
        thread::sleep(Duration::from_millis(200));

        let _ = clipboard_service.set_text("item3");
        thread::sleep(Duration::from_millis(200));

        let _ = clipboard_service.set_text("item2");
        thread::sleep(Duration::from_millis(200));

        let _ = clipboard_service.set_text("item1");
        thread::sleep(Duration::from_millis(200));

        // Create a new default stream
        let mut stream = create_default_stream().unwrap();

        // Sending the response as input should fail
        send_payload(
            &mut stream,
            payload
        );

        let recieved_payload = read_payload(&mut stream);
        
        // Cleanup
        manager.stop();
        
        // Return the recieved payload
        recieved_payload

    }

    fn check_payload_message(payload: Payload, checker: &str) {
        if let Payload::Resp(returned_response) = payload {
            assert_eq!(returned_response.message, Some(checker.to_string()));
        } else {
            panic!("Returned payload type was not correct?");
        }
    }

    fn check_payload_history(payload: Payload, checker: Vec<ClipboardItem>) {
        if let Payload::Resp(returned_response) = payload {

            match returned_response.history_snapshot {
                Some(clipboard_history) => {
                    assert_eq!(clipboard_history.get_items(), &checker);
                },
                None => {
                    panic!("Clipboard History is None.");
                },
            }

        } else {
            panic!("Returned payload type was not correct?");
        }
    }
    // ----------------------------------------------------------

    #[test]
    #[serial]
    fn test_poller_stops_on_signal() {
        // Create new manager
        let mut manager = Manager::new().unwrap();

        // start polling
        manager._polling_service(); 

        // Give time
        thread::sleep(Duration::from_millis(50));

        // Send stop signal
        manager._stop_signal.store(true, Ordering::SeqCst);

        // Give time for the poller to check the signal and exit
        thread::sleep(Duration::from_millis(200));

        // Take the handle and match
        match manager._polling_handle.take() {
            Some(p_handle) => {
                // Check if it's still running. It should not be.
                assert!(p_handle.is_finished(), "Uh-oh! Poller still running after sending the Stop Signal");

                // Join the thread.
                let _ = p_handle.join();
            },
            None => {
                panic!("POLLING HANDLE EMPTY WHEN IT SHOULD NOT HAVE BEEN!");
            }
        }

        // Close the manager
        manager.stop();
    }

    #[test]
    #[serial]
    fn test_manager_multi_spawn() {
        // Spawn a manager
        let manager = Manager::new();

        // Spawn another manager
        let err_manager = Manager::new();

        // Check if 
        match err_manager {
            Ok(_) => {
                panic!("MANAGER SHOULD NOT HAVE BEEN STARTED. MULTIPLE MANAGERS SPAWNED!")
            }
            Err(err) => {
                assert_eq!(err, DaemonError::ManagerMultiSpawn);
            }
        }

        // Close the manager
        manager.unwrap().stop();
    }

    #[test]
    #[serial]
    fn test_manager_unlock() {
        // Spawn a manager
        let mut manager: Manager = Manager::new().unwrap();

        // close the manager
        manager.stop();

        // Spawn a second manager 
        match Manager::new() {
            Ok(_) => {/* Passed */},
            Err(_) => {panic!("MANAGER DID NOT SPAWN! PREVIOUS MANAGER NOT CLEANED!")},
        };

    }

    #[test]
    #[serial]
    fn test_poller_clipboard_history_and_snapshot() {
        let recieved_payload = beam_payload(Payload::Cmd(CmdIPC::Snapshot)); // <- Should return snapshot of the history
        check_payload_history(recieved_payload, get_hopeful_history());
    }

    #[test]
    #[serial]
    fn test_invalid_ipc_command() {
        let recieved_payload = beam_payload(Payload::Resp(IPCResponse{
            history_snapshot: None,
            message: None
        }));

        check_payload_message(recieved_payload, "Wrong Payload type recieved. Expected CmdIpc but got IPCResponse.");
    }
    
    #[test]
    #[serial]
    fn test_promote_out_of_bound() {
        let recieved_payload = beam_payload(Payload::Cmd(CmdIPC::Promote(100))); // <- 100 should exceed 0... cuz history empty...
        check_payload_message(recieved_payload, "Could not promote item. Index out of bounds.");
    }

    #[test]
    #[serial]
    fn test_delete_out_of_bound() {
        let recieved_payload = beam_payload(Payload::Cmd(CmdIPC::Delete(100))); // <- 100 should exceed 0... cuz history empty...
        check_payload_message(recieved_payload, "Could not delete item. Index out of bounds.");
    }

    #[test]
    #[serial]
    fn test_promote_command() {
        let recieved_payload = beam_payload(Payload::Cmd(CmdIPC::Promote(1))); // 1,2,3,i -> 2,1,3,i

        let mut hopeful_history = get_hopeful_history();
        hopeful_history.swap(0, 1);

        check_payload_history(recieved_payload, hopeful_history);
    }

    #[test]
    #[serial]
    fn test_delete_command() {
        let recieved_payload = beam_payload(Payload::Cmd(CmdIPC::Delete(0))); // 1,2,3,i -> 2,3,i

        let mut hopeful_history = get_hopeful_history();
        hopeful_history.remove(0);

        check_payload_history(recieved_payload, hopeful_history);
    }

    #[test]
    #[serial]
    fn test_clear_command() {
        let recieved_payload = beam_payload(Payload::Cmd(CmdIPC::Clear)); // 1,2,3,i -> []
        check_payload_history(recieved_payload, vec![]);
    }
}