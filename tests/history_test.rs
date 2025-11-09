#[cfg(test)]
mod history_tests {
    use std::collections::VecDeque;

    use super_v::{common::ClipboardItem, history::ClipboardHistory};
    
    #[test]
    fn add_check() {
        // Create history
        let mut history = ClipboardHistory::new(5);

        // Sample Test Item
        let item = ClipboardItem::Text("Sample Text".to_string());
        
        // Add item to history
        history.add(item.clone());

        // Check if the history matches
        assert_eq!(history.get_items(), &VecDeque::from([item]));
    }

    #[test]
    fn pop_check() {
        // Create history
        let mut history = ClipboardHistory::new(5);

        // Create items
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        let item3 = ClipboardItem::Text("Item 3".to_string());
        let item4 = ClipboardItem::Text("Item 4".to_string());
        let item5 = ClipboardItem::Text("Item 5".to_string());
        let item6 = ClipboardItem::Text("Item 6".to_string());

        // Add items to history
        history.add(item1.clone()); // <= Oldest entry
        history.add(item2.clone());
        history.add(item3.clone());
        history.add(item4.clone());
        history.add(item5.clone());
        history.add(item6.clone());

        // Check if history auto manages the size by popping the oldest entry
        assert_eq!(history.get_items(), &VecDeque::from([item6, item5, item4, item3, item2]));
    }

    #[test]
    fn promotion() {
        // Create history
        let mut history = ClipboardHistory::new(5);
        
        // Create items
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        let item3 = ClipboardItem::Text("Item 3".to_string());

        // Add items to clipboard history
        history.add(item1.clone());
        history.add(item2.clone());
        history.add(item3.clone());

        // Promote item 2
        // This should re-order from 3->2->1 to 2->3->1
        history.promote(1);

        // Compare
        assert_eq!(history.get_items(), &VecDeque::from([item2, item3, item1]));
    }
}