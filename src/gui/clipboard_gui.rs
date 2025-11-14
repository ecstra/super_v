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
use gdk_pixbuf::{InterpType, Pixbuf};
use gtk4::{
    self as gtk,
    Application,
    prelude::*,
    gdk::Key
};
use arboard::{Clipboard, ImageData};
use std::{
    thread, 
    sync::mpsc::Sender,
    borrow::Cow, 
    rc::Rc
};

use std::collections::HashMap;
use gtk::gdk::Texture;

pub enum MainThreadMsg {
    AutoPaste,
    Close,
    DeleteItem(ClipboardItem)
}

struct GUI {
    window:           gtk::ApplicationWindow,
    stack:            gtk::Stack,
    clear_all_btn:    gtk::Button,
    search_entry:     gtk::Entry,
    items_box:        gtk::Box,
    emoji_flow_box:   gtk::FlowBox,
    image_cache: Rc<std::cell::RefCell<HashMap<Vec<u8>, Texture>>>,
    main_thread_tx:   Sender<MainThreadMsg>
}

impl GUI {
    const APP_ID: &str = "com.ecstra.super_v";

    fn new(app: &Application, main_thread_tx: Sender<MainThreadMsg>) -> Rc<Self> {
        // -------------------- Window Creation ----------------------
        let window = gtk::ApplicationWindow::builder().build();
        window.set_application(Some(app));
        // -----------------------------------------------------------

        // -------------------- Window Settings ----------------------
        const WIDTH      : i32    = 360;
        const HEIGHT     : i32    = 400;
        const TOP_PANEL  : bool   = false;
        const MODAL      : bool   = true;

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

        let stack_switcher = gtk::StackSwitcher::new();
        header_box.append(&stack_switcher);

        let clear_all_btn = gtk::Button::new();
        clear_all_btn.set_label("Clear All");
        clear_all_btn.add_css_class("clear-all-btn");
        clear_all_btn.set_hexpand(true);
        clear_all_btn.set_halign(gtk::Align::End);
        clear_all_btn.set_visible(true); // Visible by default

        header_box.append(&clear_all_btn);
        main_box.append(&header_box);

        let search_entry = gtk::Entry::new();
        search_entry.set_placeholder_text(Some("Search emojis..."));
        search_entry.add_css_class("search-entry");
        search_entry.set_visible(false); // Hidden by default
        main_box.append(&search_entry);

        // Create the Stack
        let stack = gtk::Stack::new();
        stack.set_vexpand(true);
        stack.set_hexpand(true);

        // Page 1: Clipboard
        let scrolled_window = gtk::ScrolledWindow::new();
        scrolled_window.add_css_class("scrollable-window");
        scrolled_window.set_vexpand(true);
        scrolled_window.set_hexpand(true);

        let items_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
        items_box.add_css_class("items-box");
        scrolled_window.set_child(Some(&items_box));

        stack.add_titled(&scrolled_window, Some("clipboard"), "Clipboard");
        let clipboard_page = stack.page(&scrolled_window);
        clipboard_page.set_icon_name("edit-paste-symbolic");

        // Page 2: Emoji
        let emoji_flow_box = gtk::FlowBox::new();
        emoji_flow_box.add_css_class("emoji-box");
        emoji_flow_box.set_hexpand(false);
        emoji_flow_box.set_valign(gtk::Align::Start);
        emoji_flow_box.set_vexpand(true);
        emoji_flow_box.set_max_children_per_line(7);
        emoji_flow_box.set_min_children_per_line(4);
        emoji_flow_box.set_selection_mode(gtk::SelectionMode::None);
        emoji_flow_box.set_homogeneous(true);
        emoji_flow_box.set_row_spacing(1);
        emoji_flow_box.set_column_spacing(1);

        let emoji_scrolled_window = gtk::ScrolledWindow::new();
        emoji_scrolled_window.add_css_class("scrollable-window");
        emoji_scrolled_window.set_vexpand(true);
        emoji_scrolled_window.set_hexpand(true);
        emoji_scrolled_window.set_child(Some(&emoji_flow_box));

        stack.add_titled(&emoji_scrolled_window, Some("emoji"), "Emoji");
        let emoji_page = stack.page(&emoji_scrolled_window);
        emoji_page.set_icon_name("face-smile-symbolic");

        // Final Layout Assembly
        main_box.append(&stack);
        window.set_child(Some(&main_box));
        stack_switcher.set_stack(Some(&stack));
        // ------------------------------------------------------------
        
        // Create and return the Gui instance
        Rc::new(Self {
            window: window.clone(), // Clone for the struct
            stack: stack.clone(),   // Clone for the struct
            clear_all_btn,
            search_entry,
            items_box: items_box.clone(), // Clone for the struct
            emoji_flow_box,
            image_cache: Rc::new(std::cell::RefCell::new(HashMap::new())),
            main_thread_tx
        })
    }

    fn signal_auto_paste(tx: Sender<MainThreadMsg>) {
        if let Err(err) = tx.send(MainThreadMsg::AutoPaste) {
            eprintln!("auto paste signal dropped: {err}");
        }
    }
    
    fn get_clipboard() -> Result<Clipboard, arboard::Error> {
        Clipboard::new()
    }

    fn clear_items_box(items_box: &gtk::Box) {
        while let Some(child) = items_box.first_child() {
            items_box.remove(&child);
        }
    }

    fn count_items(items_box: &gtk::Box) -> u32 {
        items_box.observe_children().n_items()
    }

    fn close_window(window: gtk::ApplicationWindow, tx: Sender<MainThreadMsg>) {
        if let Err(err) = tx.send(MainThreadMsg::Close) {
            eprintln!("close signal dropped: {err}");
        }
        window.close();
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

    fn clipboard_empty_state(items_box: &gtk::Box) {
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

    fn construct_image(width: usize, height: usize, bytes: Vec<u8>, cache: &Rc<std::cell::RefCell<HashMap<Vec<u8>, Texture>>>) -> Option<gtk::Picture> {
        const IMAGE_PREVIEW_TEXTURE_MAX_SIZE: usize = 200;
        const IMAGE_PREVIEW_DISPLAY_SIZE: i32 = 50;

        // 1. Check cache first
        if let Some(texture) = cache.borrow().get(&bytes) {
            let picture = gtk::Picture::for_paintable(texture);
            picture.set_can_shrink(true);
            picture.set_keep_aspect_ratio(true);
            picture.set_size_request(IMAGE_PREVIEW_DISPLAY_SIZE, IMAGE_PREVIEW_DISPLAY_SIZE);
            picture.set_halign(gtk::Align::Start);
            picture.add_css_class("image-preview");
            return Some(picture);
        }
        
        // 2. If not in cache, create it
        let stride = width.checked_mul(4)?;
        let expected_len = stride.checked_mul(height)?;

        let bytes_owned = gtk::glib::Bytes::from_owned(bytes[..expected_len].to_vec());
        let mut pixbuf = Pixbuf::from_bytes(
            &bytes_owned,
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            width as i32,
            height as i32,
            stride as i32,
        );

        let max_dim = width.max(height) as f32;
        if max_dim > IMAGE_PREVIEW_TEXTURE_MAX_SIZE as f32 {
            let scale = IMAGE_PREVIEW_TEXTURE_MAX_SIZE as f32 / max_dim;
            let target_width = (width as f32 * scale).round().max(1.0) as i32;
            let target_height = (height as f32 * scale).round().max(1.0) as i32;
            if let Some(resized) = pixbuf.scale_simple(target_width, target_height, InterpType::Hyper)
            {
                pixbuf = resized;
            }
        }

        let texture = gtk::gdk::Texture::for_pixbuf(&pixbuf);

        // 3. Add the new texture to the cache
        cache.borrow_mut().insert(bytes, texture.clone()); // <-- Store it

        let picture = gtk::Picture::for_paintable(&texture);
        picture.set_can_shrink(true);
        picture.set_keep_aspect_ratio(true);
        picture.set_size_request(IMAGE_PREVIEW_DISPLAY_SIZE, IMAGE_PREVIEW_DISPLAY_SIZE);
        picture.set_halign(gtk::Align::Start);
        picture.add_css_class("image-preview");
        Some(picture)
    }

    fn render_emojis(&self) {
        // Clear all widgets instantly
        while let Some(child) = self.emoji_flow_box.first_child() {
            self.emoji_flow_box.remove(&child);
        }
        
        let search_filter = self.search_entry.text().to_string();

        // 1. Get the full list of emoji strings (this is fast)
        let emojis: Vec<String> = 
            if !search_filter.trim().is_empty() {
                emojis::iter().filter(|e| e.name().contains(&search_filter) && e.as_str() != "🧑‍🩰").map(|e| e.as_str().to_string()).collect()
            } else {
                emojis::iter().filter(|e| e.as_str() != "🧑‍🩰").map(|e| e.as_str().to_string()).collect()
            };

        // 2. Wrap the list in Rc for the async loader
        let emoji_list = Rc::new(emojis);
        let progress = Rc::new(std::cell::Cell::new(0usize));
        let chunk_size = 100; // Add 100 emojis per frame

        // 3. Clone everything needed for the async task
        let emoji_flow_box = self.emoji_flow_box.clone();
        let window = self.window.clone();
        let tx = self.main_thread_tx.clone();

        // 4. Start the async loader
        gtk::glib::idle_add_local(move || {
            let start = progress.get();
            let end = (start + chunk_size).min(emoji_list.len());

            // Get the chunk of emojis to add
            if let Some(emojis_to_add) = emoji_list.get(start..end) {
                for emoji in emojis_to_add {
                    let emoji_entry = gtk::Button::with_label(emoji);
                    emoji_entry.add_css_class("emoji-btn");
                    
                    let window_clone = window.clone();
                    let tx_clone = tx.clone();
                    let emoji_str = emoji.clone(); // Clone for the closure

                    emoji_entry.connect_clicked(move |_| {
                        if let Ok(mut clipboard) = Self::get_clipboard() {
                            let emoji_str = emoji_str.clone();
                            let _ = clipboard.set_text(&emoji_str);

                            if let Err(err) = tx_clone.send(MainThreadMsg::DeleteItem(ClipboardItem::Text(emoji_str.clone()))) {
                                eprintln!("delete signal dropped: {err}");
                            }

                            Self::signal_auto_paste(tx_clone.clone());
                            Self::close_window(window_clone.clone(), tx_clone.clone());
                        }
                    });
                    emoji_flow_box.insert(&emoji_entry, -1);
                }
            }

            // Update progress
            progress.set(end);

            // If we're done, stop. Otherwise, run again.
            if end == emoji_list.len() {
                gtk::glib::ControlFlow::Break
            } else {
                gtk::glib::ControlFlow::Continue
            }
        });
    }

    fn render_clipboard_items(&self) {
        let history = Self::fetch_history();
        let items = history.get_items();

        // Clear all items
        // much easier to just clear and update
        // Than to manage the items in GUI and re-order
        Self::clear_items_box(&self.items_box);

        // Check if it's empty
        if items.is_empty() {
            Self::clipboard_empty_state(&self.items_box);
            return;
        }
        
        for item in items.iter() {
            let item_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            item_box.add_css_class("clipboard-item");

            let content_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
            content_box.set_hexpand(true);

            match item {
                ClipboardItem::Text(text) => {
                    let preview = if text.len() > 60 {
                        format!("{}...", &text[..60])
                    } else {
                        text.clone()
                    };

                    let content_label = gtk::Label::new(Some(&preview));
                    content_label.set_valign(gtk::Align::Center);
                    content_label.add_css_class("content-label");
                    content_label.set_xalign(0.0);
                    content_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    content_label.set_max_width_chars(40);

                    content_box.append(&content_label);
                }
                ClipboardItem::Image { width, height, bytes } => {
                    // Replace with image preview
                    if let Some(picture) = Self::construct_image(*width, *height, bytes.clone(), &self.image_cache) {
                        content_box.append(&picture);
                    } else {
                        let preview = format!("Image: {width} x {height}");
                        let content_label = gtk::Label::new(Some(&preview));
                        content_label.set_valign(gtk::Align::Center);
                        content_label.add_css_class("content-label");
                        content_label.set_xalign(0.0);
                        content_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                        content_label.set_max_width_chars(40);
                        content_box.append(&content_label);
                    }
                }
            }

            // Make each item clickable
            let gesture = gtk::GestureClick::new();
            let item_clone = item.clone();
            let window_clone = self.window.clone();
            let tx = self.main_thread_tx.clone();
            
            gesture.connect_released(move |_, _, _, _| {

                if let ClipboardItem::Text(text) = &item_clone {
                    if let Ok(mut clipboard) = Self::get_clipboard() {
                        if !text.trim().is_empty() {
                            // Update system clipboard
                            // This says I'm dropping the clipboard too fast (5ms)
                            // eh... should be just fine.
                            let _ = clipboard.set_text(text);

                            // Signal for auto paste and close the window
                            Self::signal_auto_paste(tx.clone());
                            Self::close_window(window_clone.clone(), tx.clone());
                            return;
                        }
                    }
                }

                if let ClipboardItem::Image {width, height, bytes} = &item_clone {

                    if let Ok(mut clipboard) = Self::get_clipboard() {
                        if !bytes.is_empty() {
                            // Same 5ms drop here...
                            let _ = clipboard.set_image(
                                ImageData {
                                    width: *width,
                                    height: *height,
                                    bytes: Cow::from(bytes)
                                }
                            );

                            // Signal for auto paste and close the window
                            Self::signal_auto_paste(tx.clone());
                            Self::close_window(window_clone.clone(), tx.clone());
                            return;
                        }
                    }
                }

                // Close the window
                Self::close_window(window_clone.clone(), tx.clone());
            });
            
            item_box.add_controller(gesture);

            // Delete button for each item
            let delete_btn = gtk::Button::new();
            delete_btn.set_icon_name("user-trash-symbolic");
            delete_btn.add_css_class("delete-btn");
            delete_btn.set_valign(gtk::Align::Start);

            // Make the delete button functional
            let items_box = self.items_box.clone();
            let item_box_clone = item_box.clone();

            delete_btn.connect_clicked(move |_| {
                // Get the index of the object
                let current_index = (0..Self::count_items(&items_box))
                    .find(|&i| {
                        items_box
                            .observe_children() // Get all children
                            .item(i) // Position
                            .and_then(|obj| obj.downcast::<gtk::Box>().ok()) // Convert object -> Box
                            .as_ref() // Ref, no need to move
                            == Some(&item_box_clone) // Matches the item we want to delete?
                    })
                    .unwrap_or(0) as usize;

                // Remove that object from items box
                items_box.remove(&item_box_clone);
                
                // If box is empty after deleting, add the "empty" message
                if items_box.first_child().is_none() {
                    Self::clipboard_empty_state(&items_box);   
                }

                // Send command to update the server
                thread::spawn(move || {
                    send_command(CmdIPC::Delete(current_index));
                });
            });

            item_box.append(&content_box);
            item_box.append(&delete_btn);

            self.items_box.append(&item_box);
        }
    }

    /// Handles logic for when the active tab (Stack page) changes.
    fn handle_tab_switch(&self, stack: &gtk::Stack) {
        if let Some(name) = stack.visible_child_name() {
            let is_clipboard = name == "clipboard";

            // Toggle visibility of page-specific controls
            self.clear_all_btn.set_visible(is_clipboard);
            self.search_entry.set_visible(!is_clipboard);

            // Call the appropriate render function
            if is_clipboard {
                self.render_clipboard_items();
            } else {
                self.render_emojis();
            }
        }
    }

    /// Connects signals and presents the main window.
    /// This consumes the Rc<Self> to correctly set up closures.
    fn build(self: Rc<Self>) {
        // -------------------- Initial State -------------------------
        // Initial Clipboard render
        self.render_clipboard_items();
        // ------------------------------------------------------------

        // -------------------- Connect Events ------------------------
        let all_items = self.items_box.clone();

        // Clear all btn connector
        self.clear_all_btn.connect_clicked(move |_| {
            // Clear all items
            Self::clear_items_box(&all_items);

            // Show empty clipboard screen
            Self::clipboard_empty_state(&all_items);

            // Send message to server to clear
            thread::spawn(move || {
                send_command(CmdIPC::Clear);
            });
        });

        // Tab Switching
        // `self` is Rc<GUI>, so `self.clone()` clones the Rc
        let gui_clone_stack = self.clone(); 
        self.stack.connect_visible_child_name_notify(move |stack| {
            // Call the instance method on the cloned Rc
            gui_clone_stack.handle_tab_switch(stack);
        });

        // Quit Events
        // Quit when "esc" is pressed
        let window_clone = self.window.clone(); // Need a new clone for this closure
        let tx = self.main_thread_tx.clone();
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == Key::Escape {
                Self::close_window(window_clone.clone(), tx.clone());
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        self.window.add_controller(key_controller);

        // Quit when focus is lost
        let window_clone = self.window.clone(); // Need a new clone for this closure
        let tx = self.main_thread_tx.clone();
        self.window.connect_is_active_notify(move |window| {
            if !window.is_active() {
                Self::close_window(window_clone.clone(), tx.clone());
            }
        });
        
        // Emoji Search
        // Clone the Rc for the search entry closure
        let gui_clone_search = self.clone();
        self.search_entry.connect_changed(move |_| {
            // Re-render the emoji list every time the text changes
            gui_clone_search.render_emojis();
        });
        // -----------------------------------------------------------

        // Present the window
        self.window.present();
    }
}

pub fn send_command(cmd: CmdIPC) -> Option<ClipboardHistory> {
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

fn build_ui(app: &Application, tx: Sender<MainThreadMsg>) {
    // Create the Gui. This struct now owns all the widgets.
    // The `Rc` will keep `gui` alive as long as the closures
    // (event handlers) are alive.
    let gui = GUI::new(app, tx);
    gui.build();
}

pub fn run_gui(tx: Sender<MainThreadMsg>) {
    let app = Application::builder()
        .application_id(GUI::APP_ID)
        .build();

    app.connect_activate(move |app| {
        build_ui(app, tx.clone());
    });
    app.run_with_args(&Vec::<String>::new());
}