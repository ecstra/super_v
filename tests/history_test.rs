#[cfg(test)]
mod history_tests {
    use std::collections::VecDeque;

    use super_v::{common::{ClipboardItem, ClipboardError}, history::ClipboardHistory};
    
    #[test]
    fn test_history_add_item() {
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
    fn test_capacity_fill_and_overflow() {
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
        assert!(history.get_items().len() == 5, "Length must be 5, but got {}", history.get_items().len());
        assert_eq!(history.get_items(), &VecDeque::from([item6, item5, item4, item3, item2]));
    }

    #[test]
    fn test_item_promotion() {
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
        history.promote(1).unwrap();

        // Compare
        assert_eq!(history.get_items(), &VecDeque::from([item2, item3, item1]));
    }

    #[test]
    #[should_panic]
    fn test_promote_out_of_bounds_panic() {
        // Create history with items
        let mut history = ClipboardHistory::new(5);
        
        let item1 = ClipboardItem::Text("Item 1".to_string());
        history.add(item1);
        
        // Try to promote an index that doesn't exist (should panic)
        history.promote(5).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_promote_empty_history_panic() {
        // Create empty history
        let mut history = ClipboardHistory::new(5);
        
        // Try to promote from empty history (should panic)
        history.promote(0).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_promote_negative_bounds_panic() {
        // Create history with items
        let mut history = ClipboardHistory::new(5);
        
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        history.add(item1);
        history.add(item2);
        
        // Try to promote at exactly the length (should panic)
        // If we have 2 items (indices 0,1), trying to access index 2 should panic
        history.promote(2).unwrap();
    }

    #[test]
    fn test_zero_capacity_history() {
        // Test edge case: history with 0 capacity
        let mut history = ClipboardHistory::new(0);
        
        let item1 = ClipboardItem::Text("Item 1".to_string());
        history.add(item1.clone());
        
        // With max_size of 0, the item should be added then immediately removed
        // since len (1) > max_size (0)
        assert_eq!(history.get_items().len(), 0);
    }

    #[test]
    fn test_large_image_data() {
        // Test with large image data to ensure no memory issues
        let mut history = ClipboardHistory::new(3);
        
        // Create a large image (1MB of data)
        let large_bytes = vec![0u8; 1_000_000];
        let large_image = ClipboardItem::Image {
            width: 1000,
            height: 1000,
            bytes: large_bytes,
        };
        
        history.add(large_image.clone());
        
        assert_eq!(history.get_items().len(), 1);
        assert_eq!(history.get_items().front(), Some(&large_image));
    }

    #[test]
    fn test_empty_text_item() {
        // Test edge case: empty string
        let mut history = ClipboardHistory::new(5);
        
        let empty_text = ClipboardItem::Text(String::new());
        history.add(empty_text.clone());
        
        assert_eq!(history.get_items().len(), 1);
        assert_eq!(history.get_items(), &VecDeque::from([empty_text]));
    }

    #[test]
    fn test_very_long_text_item() {
        // Test with very long text
        let mut history = ClipboardHistory::new(5);
        
        let long_text = "A".repeat(100_000);
        let long_item = ClipboardItem::Text(long_text.clone());
        history.add(long_item.clone());
        
        assert_eq!(history.get_items().len(), 1);
        if let ClipboardItem::Text(ref text) = history.get_items()[0] {
            assert_eq!(text.len(), 100_000);
        } else {
            panic!("Expected Text item");
        }
    }

    #[test]
    fn test_special_characters_in_text() {
        // Test with special characters and unicode
        let mut history = ClipboardHistory::new(5);
        
        let special_text = ClipboardItem::Text("Hello 世界! 🦀 \n\t\r".to_string());
        history.add(special_text.clone());
        
        assert_eq!(history.get_items().len(), 1);
        assert_eq!(history.get_items(), &VecDeque::from([special_text]));
    }

    #[test]
    fn test_zero_dimension_image() {
        // Test edge case: image with zero dimensions
        let mut history = ClipboardHistory::new(5);
        
        let zero_img = ClipboardItem::Image {
            width: 0,
            height: 0,
            bytes: vec![],
        };
        
        history.add(zero_img.clone());
        
        assert_eq!(history.get_items().len(), 1);
        assert_eq!(history.get_items(), &VecDeque::from([zero_img]));
    }

    #[test]
    fn test_multiple_promote_same_pos() {
        // Test promoting the same item multiple times
        let mut history = ClipboardHistory::new(5);
        
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        let item3 = ClipboardItem::Text("Item 3".to_string());
        
        history.add(item1.clone());
        history.add(item2.clone());
        history.add(item3.clone());
        
        // Promote item at index 2 multiple times
        history.promote(2).unwrap(); // 3,2,1 -> 1,3,2
        assert_eq!(history.get_items(), &VecDeque::from([item1.clone(), item3.clone(), item2.clone()]));
        
        history.promote(2).unwrap(); // 1,3,2 -> 2,1,2
        assert_eq!(history.get_items(), &VecDeque::from([item2.clone(), item1.clone(), item3.clone()]));
    }

    #[test]
    fn test_duplicate_handling() {
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
    fn test_clear_history() {
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
    fn test_empty_history_operations() {
        let history = ClipboardHistory::new(5);
        assert_eq!(history.get_items().len(), 0);
    }

    #[test]
    fn test_image_items() {
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
    fn test_mixed_content_types() {
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
    fn test_promote_first_item() {
        // Create history
        let mut history = ClipboardHistory::new(3);
        
        // Create items
        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        
        // Add items to history
        history.add(item1.clone());
        history.add(item2.clone());
        
        // Promote first item (index 0) - should remain at top
        history.promote(0).unwrap();
        
        assert_eq!(history.get_items(), &VecDeque::from([item2, item1]));
    }

    #[test]
    fn test_delete_item_success() {
        // Create history and populate with three entries
        let mut history = ClipboardHistory::new(3);

        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());
        let item3 = ClipboardItem::Text("Item 3".to_string());

        history.add(item1.clone());
        history.add(item2.clone());
        history.add(item3.clone());

        // Delete the middle item (current order: 3,2,1)
        let result = history.delete(1);
        assert_eq!(result, Ok(()));
        assert_eq!(history.get_items(), &VecDeque::from([item3, item1]));
    }

    #[test]
    fn test_delete_out_of_bounds_error() {
        // Attempt to delete from an empty history should yield an out-of-bounds error
        let mut history = ClipboardHistory::new(2);
        assert_eq!(history.delete(0), Err(ClipboardError::IndexOutOfBound));

        // Populate history and try to delete past the end
        history.add(ClipboardItem::Text("Item".to_string()));
        assert_eq!(history.delete(5), Err(ClipboardError::IndexOutOfBound));
    }

    #[test]
    fn test_delete_this_item_success() {
        // Create history
        let mut history = ClipboardHistory::new(3);

        let item1 = ClipboardItem::Text("Item 1".to_string());
        let item2 = ClipboardItem::Text("Item 2".to_string());

        history.add(item1.clone());
        history.add(item2.clone());

        // Delete a specific item by value
        let result = history.delete_this(item1.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(history.get_items(), &VecDeque::from([item2]));
    }

    #[test]
    #[should_panic]
    fn test_delete_this_missing_item_panic() {
        // Create history with a single entry and attempt to delete an unknown item
        let mut history = ClipboardHistory::new(2);
        history.add(ClipboardItem::Text("Known".to_string()));

        // delete_this uses unwrap on the internal search, so a missing item should panic
        history.delete_this(ClipboardItem::Text("Missing".to_string())).unwrap();
    }
}