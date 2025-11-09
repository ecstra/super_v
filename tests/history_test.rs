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
        assert_eq!(history.get_items().len(), 5);
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

    #[test]
    fn duplicate_handling() {
        // Create history
        let mut history = ClipboardHistory::new(5);
        
        // Create items
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        
        // Add items to clipboard history
        history.add(item1.clone());
        history.add(item2.clone());
        history.add(item1.clone()); // <= Duplicates should be promoted to top
        
        // Should have only 2 items with item1 promoted to front
        assert_eq!(history.get_items().len(), 2);
        assert_eq!(history.get_items(), &VecDeque::from([item1, item2]));
    }

    #[test]
    fn clear_history() {
        // Create history
        let mut history = ClipboardHistory::new(5);
        
        // Create items
        history.add(ClipboardItem::Text("Item 1".to_string()));
        history.add(ClipboardItem::Text("Item 2".to_string()));
        
        // Clear the history
        history.clear();
        
        // Check if any elements exist after clearing
        assert_eq!(history.get_items().len(), 0);
    }

    #[test]
    fn empty_history_operations() {
        let history = ClipboardHistory::new(5);
        assert_eq!(history.get_items().len(), 0);
    }

    #[test]
    fn image_items() {
        // Create history
        let mut history = ClipboardHistory::new(3);
        
        // Create Image items (Random data)
        let image1 = ClipboardItem::Image {
            width: 100,
            height: 100,
            bytes: vec![0u8; 100],
        };
        
        let image2 = ClipboardItem::Image {
            width: 200,
            height: 150,
            bytes: vec![255u8; 200],
        };
        
        // Add images
        history.add(image1.clone());
        history.add(image2.clone());
        
        // Check if images were added
        assert_eq!(history.get_items(), &VecDeque::from([image2, image1]));
    }

    #[test]
    fn mixed_content_types() {
        // Create history
        let mut history = ClipboardHistory::new(5);
        
        // Create items, one of each type
        let text = ClipboardItem::Text("Hello".to_string());
        let image = ClipboardItem::Image {
            width: 50,
            height: 50,
            bytes: vec![0u8; 50],
        };
        
        // Add items to history
        history.add(text.clone());
        history.add(image.clone());
        
        // Check history
        assert_eq!(history.get_items(), &VecDeque::from([image, text]));
    }

    #[test]
    fn promote_first_item() {
        // Create history
        let mut history = ClipboardHistory::new(3);
        
        // Create items
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        
        // Add items to history
        history.add(item1.clone());
        history.add(item2.clone());
        
        // Promote first item (index 0) - should remain at top
        history.promote(0);
        
        assert_eq!(history.get_items(), &VecDeque::from([item2, item1]));
    }

    #[test]
    fn promote_last_item() {
        // Create history
        let mut history = ClipboardHistory::new(5);
        
        // Create items
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        let item3 = ClipboardItem::Text("Item 3".to_string());
        
        // Add items to history
        history.add(item1.clone());
        history.add(item2.clone());
        history.add(item3.clone());
        
        // Promote last item (index 2)
        history.promote(2);
        
        assert_eq!(history.get_items(), &VecDeque::from([item1, item3, item2]));
    }

    #[test]
    fn single_capacity_history() {
        // Create history
        let mut history = ClipboardHistory::new(1);
        
        // Create items
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        
        // Add items to history
        history.add(item1.clone());
        history.add(item2.clone());
        
        // Should only keep the latest item
        assert_eq!(history.get_items().len(), 1);
        assert_eq!(history.get_items(), &VecDeque::from([item2]));
    }
}