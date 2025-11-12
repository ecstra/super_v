use crate::{
    common::ClipboardItem, 
    history::ClipboardHistory, 
    services::{
        clipboard_ipc_server::{
            CmdIPC,
            IPCRequest,
            Payload,
            create_default_stream,
            read_payload,
            send_payload
        }
    }
};

use gtk4::{
    self as gtk,
    Application,
    prelude::*,
    gdk::Key
};
use arboard::Clipboard;
use std::{
    fs,
    sync::{
        Arc, 
        Mutex
    }, time::Duration
};

const APP_ID: &str = "com.ecstra.super_v";
pub const SHOULD_PASTE_FLAG: &str = "/tmp/super_v_should_paste";

fn append_empty_state(items_box: &gtk::Box) {
    let empty_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    empty_box.set_valign(gtk::Align::Center);
    empty_box.set_vexpand(true);
    empty_box.set_margin_top(-10);

    let empty_title = gtk::Label::new(Some("Clipboard empty"));
    empty_title.add_css_class("empty-title");

    let empty_subtitle = gtk::Label::new(Some("Copy something and come back here"));
    empty_subtitle.add_css_class("empty-subtitle");

    empty_box.append(&empty_title);
    empty_box.append(&empty_subtitle);
    items_box.append(&empty_box);
}

pub fn run_gui() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run_with_args(&Vec::<String>::new()); // Changed from app.run()
}

fn fetch_history() -> ClipboardHistory {
    let new_clipboard = ClipboardHistory::new(25);

    match create_default_stream() {
        Ok(mut stream) => {
            send_payload(&mut stream, Payload::Request(IPCRequest {
                cmd: CmdIPC::Snapshot
            }));
            
            let received_payload = read_payload(&mut stream);
            match received_payload {
                Payload::Response(ipc_resp) => {
                    ipc_resp.history_snapshot.unwrap_or_else(|| new_clipboard)
                }
                _ => new_clipboard
            }
        }
        Err(_) => new_clipboard
    }
}

fn send_command(cmd: CmdIPC) -> Option<ClipboardHistory> {
    match create_default_stream() {
        Ok(mut stream) => {
            send_payload(&mut stream, Payload::Request(IPCRequest { cmd }));
            
            let received_payload = read_payload(&mut stream);
            if let Payload::Response(ipc_resp) = received_payload {
                return ipc_resp.history_snapshot;
            }
            None
        }
        Err(_) => None,
    }
}

fn refresh_items(
    items_box: &gtk::Box, 
    window: &gtk::ApplicationWindow, 
    persistent_clipboard: Arc<Mutex<Clipboard>>
) {
    // Clear all existing items
    while let Some(child) = items_box.first_child() {
        items_box.remove(&child);
    }

    // Fetch fresh history
    let fetched_history = fetch_history();
    let clipboard_items = fetched_history.get_items();

    // Show empty state if no items
    if clipboard_items.is_empty() {
        append_empty_state(items_box);
        return;
    }

    // Rebuild items
    for (_, item) in clipboard_items.iter().enumerate() {
        let revealer = gtk::Revealer::new();
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);
        revealer.set_transition_duration(220);
        revealer.set_reveal_child(true);

        let item_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        item_box.add_css_class("clipboard-item");

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        content_box.set_hexpand(true);

        let content_preview = match item {
            ClipboardItem::Text(text) => {
                if text.len() > 60 {
                    format!("{}...", &text[..60])
                } else {
                    text.clone()
                }
            }
            ClipboardItem::Image { width, height, .. } => {
                format!("{}x{}", width, height)
            }
        };

        let content_label = gtk::Label::new(Some(&content_preview));
        content_label.set_valign(gtk::Align::Center);
        content_label.add_css_class("content-label");
        content_label.set_xalign(0.0);
        content_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        content_label.set_max_width_chars(40);

        content_box.append(&content_label);

        // Make the item clickable
        let gesture = gtk::GestureClick::new();
        let window_clone = window.clone();
        let item_clone = item.clone();
        let clipboard_arc = persistent_clipboard.clone();
        
        // In gesture.connect_released (item clicked):
        gesture.connect_released(move |_, _, _, _| {
            let item_for_thread = item_clone.clone();
            let clipboard_for_thread = clipboard_arc.clone();
            let window_for_close = window_clone.clone();
            
            if let ClipboardItem::Text(text) = &item_for_thread {
                if let Ok(mut clipboard) = clipboard_for_thread.lock() {
                    if !text.trim().is_empty() {
                        let _ = clipboard.set_text(text);
                        // Create flag file to signal paste should happen
                        let _ = fs::write(SHOULD_PASTE_FLAG, "1");
                        window_for_close.close();
                        return;
                    }
                }
            }
            
            window_clone.close();
        });
        
        item_box.add_controller(gesture);

        // Delete button with trash icon
        let delete_btn = gtk::Button::new();
        delete_btn.set_icon_name("user-trash-symbolic");
        delete_btn.add_css_class("delete-btn");
        delete_btn.set_valign(gtk::Align::Start);

        // Delete button click handler
        let items_box_clone = items_box.clone();
        let item_revealer = revealer.clone();

        delete_btn.connect_clicked(move |_| {
            let current_index = (0..items_box_clone.observe_children().n_items())
                .find(|&i| {
                    items_box_clone
                        .observe_children()
                        .item(i)
                        .and_then(|obj| obj.downcast::<gtk::Revealer>().ok())
                        .as_ref() == Some(&item_revealer)
                })
                .unwrap_or(0);

            item_revealer.set_reveal_child(false);

            let items_box_for_removal = items_box_clone.clone();
            let revealer_for_removal = item_revealer.clone();

            gtk::glib::timeout_add_local_once(Duration::from_millis(220), move || {
                items_box_for_removal.remove(&revealer_for_removal);

                if items_box_for_removal.first_child().is_none() {
                    append_empty_state(&items_box_for_removal);
                }

                std::thread::spawn(move || {
                    send_command(CmdIPC::Delete(current_index as usize));
                });
            });
        });

        item_box.append(&content_box);
        item_box.append(&delete_btn);

        revealer.set_child(Some(&item_box));
        items_box.append(&revealer);
    }
}

fn show_emojis(
    items_box: &gtk::Box,
    window: &gtk::ApplicationWindow,
    persistent_clipboard: Arc<Mutex<Clipboard>>,
    search_filter: Option<&str>,
) {
    // Clear all existing items
    while let Some(child) = items_box.first_child() {
        items_box.remove(&child);
    }

    // Create a FlowBox for grid layout
    let flow_box = gtk::FlowBox::new();
    flow_box.set_hexpand(false);
    flow_box.set_valign(gtk::Align::Start);
    flow_box.set_max_children_per_line(8);
    flow_box.set_min_children_per_line(4);
    flow_box.set_selection_mode(gtk::SelectionMode::None);
    flow_box.set_homogeneous(true);
    flow_box.set_row_spacing(1);
    flow_box.set_column_spacing(1);

    // Filter emojis based on search
    let emoji_iter: Vec<&emojis::Emoji> = if let Some(filter) = search_filter {
        let filter_lower = filter.to_lowercase();
        emojis::iter()
            .filter(|e| {
                // Filter out problematic multi-width emoji
                if e.as_str() == "🧑‍🩰" {
                    return false;
                }
                e.name().to_lowercase().contains(&filter_lower) ||
                e.shortcode().map(|s| s.to_lowercase().contains(&filter_lower)).unwrap_or(false)
            })
            .collect()
    } else {
        emojis::iter()
            .filter(|e| e.as_str() != "🧑‍🩰") // Filter out problematic multi-width emoji
            .collect()
    };

    for emoji in emoji_iter {
        // let emoji = emoji_iter[0];
        let emoji_btn = gtk::Button::new();
        emoji_btn.set_label(emoji.as_str());
        emoji_btn.add_css_class("emoji-btn");
        
        let emoji_str = emoji.as_str().to_string();
        let window_clone = window.clone();
        let clipboard_clone = persistent_clipboard.clone();
        
        emoji_btn.connect_clicked(move |_| {
            // Copy emoji to clipboard
            if let Ok(mut clipboard) = clipboard_clone.lock() {
                let _ = clipboard.set_text(&emoji_str);
            }
            
            // Delete first clipboard item (position 0) so emoji isn't stored
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(120)); // <- Wait for the emoji to be picked up by the daemon
                send_command(CmdIPC::Delete(0));
            });
            
            // Create flag file to signal paste should happen
            let _ = fs::write(SHOULD_PASTE_FLAG, "1");
            
            // Close window
            window_clone.close();
        });
        
        flow_box.insert(&emoji_btn, -1);
    }
    
    items_box.append(&flow_box);
}

fn build_ui(app: &Application) {
    // -------------------- Window Creation ----------------------
    let window = gtk::ApplicationWindow::builder().build();
    window.set_application(Some(app)); // <- Window is assigned to our main application
    let persistent_clipboard = Arc::new(
        Mutex::new(
            Clipboard::new().unwrap()
    ));
    // -----------------------------------------------------------


    // -------------------- Window Settings ----------------------
    // Changables
    const WIDTH      : i32    = 360;
    const HEIGHT     : i32    = 400;

    // Flags
    const TOP_PANEL  : bool   = false;
    const MODAL      : bool   = true;

    // Apply the settings
    window.set_default_size(WIDTH, HEIGHT);

    window.set_decorated(TOP_PANEL);
    window.set_modal(MODAL);
    // -----------------------------------------------------------


    // ------------------------ CSS ------------------------------
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(include_str!("./style.css"));

    gtk::style_context_add_provider_for_display(
        &WidgetExt::display(&window),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    // -----------------------------------------------------------


   // --------------------- Main Layout --------------------------
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.add_css_class("main-box");

    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header_box.add_css_class("header-box");

    let clipboard_tab = gtk::Button::new();
    clipboard_tab.set_icon_name("edit-paste-symbolic");
    clipboard_tab.add_css_class("tab-button");
    clipboard_tab.add_css_class("active-tab");

    let emoji_tab = gtk::Button::new();
    emoji_tab.set_icon_name("face-smile-symbolic");
    emoji_tab.add_css_class("tab-button");

    header_box.append(&clipboard_tab);
    header_box.append(&emoji_tab);

    let clear_all_btn = gtk::Button::new();
    clear_all_btn.set_label("Clear All");
    clear_all_btn.add_css_class("clear-all-btn");
    clear_all_btn.set_hexpand(true);
    clear_all_btn.set_halign(gtk::Align::End);

    header_box.append(&clear_all_btn);
    main_box.append(&header_box);

    // Search box (for emojis)
    let search_entry = gtk::Entry::new();
    search_entry.set_placeholder_text(Some("Search emojis..."));
    search_entry.add_css_class("search-entry");
    search_entry.set_visible(false); // Hidden by default
    main_box.append(&search_entry);

    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.add_css_class("scrollable-window");
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);

    let items_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
    items_box.add_css_class("items-box");

    scrolled_window.set_child(Some(&items_box));
    main_box.append(&scrolled_window);
    window.set_child(Some(&main_box));
    // -----------------------------------------------------------

    // Initial items load
    refresh_items(&items_box, &window, persistent_clipboard.clone());

    // Tab switching
    let items_box_clip = items_box.clone();
    let window_clip = window.clone();
    let clipboard_clip = persistent_clipboard.clone();
    let emoji_tab_clip = emoji_tab.clone();
    let search_entry_clip = search_entry.clone();
    let clear_all_clip = clear_all_btn.clone();
    clipboard_tab.connect_clicked(move |btn| {
        btn.add_css_class("active-tab");
        emoji_tab_clip.remove_css_class("active-tab");
        search_entry_clip.set_visible(false);
        clear_all_clip.set_visible(true);
        refresh_items(&items_box_clip, &window_clip, clipboard_clip.clone());
    });

    let items_box_emoji = items_box.clone();
    let window_emoji = window.clone();
    let clipboard_emoji = persistent_clipboard.clone();
    let clipboard_tab_emoji = clipboard_tab.clone();
    let search_entry_emoji = search_entry.clone();
    let clear_all_emoji = clear_all_btn.clone();
    emoji_tab.connect_clicked(move |btn| {
        btn.add_css_class("active-tab");
        clipboard_tab_emoji.remove_css_class("active-tab");
        search_entry_emoji.set_visible(true);
        clear_all_emoji.set_visible(false);
        show_emojis(&items_box_emoji, &window_emoji, clipboard_emoji.clone(), None);
    });

    // Emoji search
    let items_box_search = items_box.clone();
    let window_search = window.clone();
    let clipboard_search = persistent_clipboard.clone();
    search_entry.connect_changed(move |entry| {
        let search_text = entry.text().to_string().to_lowercase();
        let filter = if search_text.is_empty() {
            None
        } else {
            Some(search_text)
        };
        show_emojis(&items_box_search, &window_search, clipboard_search.clone(), filter.as_deref());
    });

    // Clear All button handler
    let items_box_clear = items_box.clone();
    let window_clear = window.clone();
    let clipboard_clear = persistent_clipboard.clone();
    clear_all_btn.connect_clicked(move |_| {
        let observer = items_box_clear.observe_children();
        let mut revealers: Vec<gtk::Revealer> = Vec::new();

        for idx in 0..observer.n_items() {
            if let Some(obj) = observer.item(idx).and_then(|o| o.downcast::<gtk::Revealer>().ok()) {
                revealers.push(obj);
            }
        }

        if revealers.is_empty() {
            std::thread::spawn(|| {
                send_command(CmdIPC::Clear);
            });
            refresh_items(&items_box_clear, &window_clear, clipboard_clear.clone());
            return;
        }

    let original_spacing = items_box_clear.spacing();
    items_box_clear.set_spacing(0);

        for (idx, revealer) in revealers.iter().enumerate() {
            let revealer_clone = revealer.clone();
            let delay = (idx as u64) * 16;
            gtk::glib::timeout_add_local_once(Duration::from_millis(delay), move || {
                revealer_clone.set_reveal_child(false);
            });
        }

        let total_delay = 240 + (revealers.len() as u64 * 16);
        let items_box_after = items_box_clear.clone();
        let window_after = window_clear.clone();
        let clipboard_after = clipboard_clear.clone();
        let spacing_restore = original_spacing;

        gtk::glib::timeout_add_local_once(Duration::from_millis(total_delay), move || {
            while let Some(child) = items_box_after.first_child() {
                items_box_after.remove(&child);
            }

            items_box_after.set_spacing(spacing_restore);

            std::thread::spawn(|| {
                send_command(CmdIPC::Clear);
            });

            append_empty_state(&items_box_after);

            let items_box_refresh = items_box_after.clone();
            let window_refresh = window_after.clone();
            let clipboard_refresh = clipboard_after.clone();
            let spacing_refresh = spacing_restore;

            gtk::glib::timeout_add_local_once(Duration::from_millis(60), move || {
                items_box_refresh.set_spacing(spacing_refresh);
                refresh_items(&items_box_refresh, &window_refresh, clipboard_refresh.clone());
            });
        });
    });

    // ---------------------- Quit Events ------------------------
    let key_controller = gtk::EventControllerKey::new();
    let window_esc = window.clone();

    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == Key::Escape {
            window_esc.close();
            gtk::glib::Propagation::Stop
        } else {
            gtk::glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    window.connect_is_active_notify(move |window| {
        if !window.is_active() {
            window.close()
        }
    });
    // -----------------------------------------------------------

    window.present();
}