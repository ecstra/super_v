// Standard Crates
//..

// External Crates
use arboard::{Clipboard};

// My Crates
use super_v::services::clipboard_monitor::Monitor;

/*
Notes:
- Add tests
- STOP USING UNWRAP
- Threaded Clipboard Manager that has a polling mechanism and manages history.
- Keyboard simulation using rdev (to paste when item is clicked)
- Mouse pointer monitoring to open the window at cursor location
- Emoji screen
*/

// ----------------------------- Main --------------------------------
fn main() {
    // let mut ch = ClipboardHistory::new(20);
    // ch.add(ClipboardItem::Text("miaow".to_string()));
    // ch.add(ClipboardItem::Text("woof".to_string()));
    // ch.add(ClipboardItem::Text("rawr".to_string()));
    // ch.promote(1);
    // println!("{ch}");

    let clipboard = Clipboard::new().unwrap();
    clipboard.monitor();

    // let mut clipboard = Clipboard::new().unwrap();
    // println!("{}", clipboard.get_item().unwrap());
}
// -------------------------------------------------------------------