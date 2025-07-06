use bytes::BufMut;

use crate::{DataOffset, SIZEOF_U16};

pub struct MetaEntryOffset {
    offsets: Vec<DataOffset>,
}

impl MetaEntryOffset {
    pub fn new(initial: Option<Vec<DataOffset>>) -> Self {
        MetaEntryOffset {
            offsets: initial.unwrap_or_else(|| vec![]),
        }
    }

    pub fn add(&mut self, offset: DataOffset) {
        self.offsets.push(offset);
    }

    // ------------------------------------------------------------------
    // |   offset (u16)   | ... |    offset (u16)   |    length (u16)   |
    // ------------------------------------------------------------------
    pub fn write_to(&self, buffer: &mut Vec<u8>) {
        for &offset in &self.offsets {
            buffer.put_u16(offset);
        }
        buffer.put_u16(self.offsets.len() as DataOffset);
    }

    pub fn len(&self) -> usize {
        SIZEOF_U16 + self.offsets.len() * SIZEOF_U16 // 2 bytes for count + 2 bytes for each offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_none() {
        let entry_offset = MetaEntryOffset::new(None);
        assert_eq!(entry_offset.offsets.len(), 0);
        assert_eq!(entry_offset.len(), 2); // Just the count field
    }

    #[test]
    fn test_new_with_initial_values() {
        let initial = vec![10, 20, 30];
        let entry_offset = MetaEntryOffset::new(Some(initial.clone()));
        assert_eq!(entry_offset.offsets, initial);
        assert_eq!(entry_offset.len(), 2 + 3 * 2); // count + 3 offsets
    }

    #[test]
    fn test_add_single_offset() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(100);
        assert_eq!(entry_offset.offsets.len(), 1);
        assert_eq!(entry_offset.offsets[0], 100);
        assert_eq!(entry_offset.len(), 4); // count + 1 offset
    }

    #[test]
    fn test_add_multiple_offsets() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(100);
        entry_offset.add(200);
        entry_offset.add(300);
        
        assert_eq!(entry_offset.offsets.len(), 3);
        assert_eq!(entry_offset.offsets, vec![100, 200, 300]);
        assert_eq!(entry_offset.len(), 8); // count + 3 offsets
    }

    #[test]
    fn test_size_calculation() {
        let mut entry_offset = MetaEntryOffset::new(None);
        
        // Empty: 2 bytes for count
        assert_eq!(entry_offset.len(), 2);
        
        // One offset: 2 bytes for count + 2 bytes for offset
        entry_offset.add(50);
        assert_eq!(entry_offset.len(), 4);
        
        // Two offsets: 2 bytes for count + 4 bytes for offsets
        entry_offset.add(75);
        assert_eq!(entry_offset.len(), 6);
        
        // Ten offsets: 2 bytes for count + 20 bytes for offsets
        for i in 0..8 {
            entry_offset.add(i * 10);
        }
        assert_eq!(entry_offset.len(), 22);
    }

    #[test]
    fn test_finish_empty() {
        let entry_offset = MetaEntryOffset::new(None);
        let mut buffer = Vec::new();
        
        entry_offset.write_to(&mut buffer);
        
        assert_eq!(buffer.len(), 2);
        // Should contain count of 0
        assert_eq!(DataOffset::from_be_bytes([buffer[0], buffer[1]]), 0);
    }

    #[test]
    fn test_finish_single_offset() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(1024);
        let mut buffer = Vec::new();
        
        entry_offset.write_to(&mut buffer);
        
        assert_eq!(buffer.len(), 4);
        // Offset should be 1024
        assert_eq!(DataOffset::from_be_bytes([buffer[0], buffer[1]]), 1024);
        // Count should be 1
        assert_eq!(DataOffset::from_be_bytes([buffer[2], buffer[3]]), 1);
    }

    #[test]
    fn test_finish_multiple_offsets() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(256);
        entry_offset.add(512);
        entry_offset.add(1024);
        let mut buffer = Vec::new();
        
        entry_offset.write_to(&mut buffer);
        
        assert_eq!(buffer.len(), 8); // 6 bytes offsets + 2 bytes count
        
        // First offset: 256
        assert_eq!(DataOffset::from_be_bytes([buffer[0], buffer[1]]), 256);
        // Second offset: 512
        assert_eq!(DataOffset::from_be_bytes([buffer[2], buffer[3]]), 512);
        // Third offset: 1024
        assert_eq!(DataOffset::from_be_bytes([buffer[4], buffer[5]]), 1024);
        // Count should be 3
        assert_eq!(DataOffset::from_be_bytes([buffer[6], buffer[7]]), 3);
    }

    #[test]
    fn test_finish_with_initial_values() {
        let initial = vec![100, 200, 300];
        let entry_offset = MetaEntryOffset::new(Some(initial));
        let mut buffer = Vec::new();
        
        entry_offset.write_to(&mut buffer);
        
        assert_eq!(buffer.len(), 8);
        // Check each offset
        assert_eq!(DataOffset::from_be_bytes([buffer[0], buffer[1]]), 100);
        assert_eq!(DataOffset::from_be_bytes([buffer[2], buffer[3]]), 200);
        assert_eq!(DataOffset::from_be_bytes([buffer[4], buffer[5]]), 300);
        // Count should be 3
        assert_eq!(DataOffset::from_be_bytes([buffer[6], buffer[7]]), 3);
    }

    #[test]
    fn test_finish_appends_to_existing_buffer() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(42);
        
        let mut buffer = vec![0xFF, 0xFE]; // Pre-existing data
        entry_offset.write_to(&mut buffer);
        
        assert_eq!(buffer.len(), 6); // 2 existing + 2 offset + 2 count
        // Pre-existing data should remain
        assert_eq!(buffer[0], 0xFF);
        assert_eq!(buffer[1], 0xFE);
        // New data appended
        assert_eq!(DataOffset::from_be_bytes([buffer[2], buffer[3]]), 42);
        assert_eq!(DataOffset::from_be_bytes([buffer[4], buffer[5]]), 1);
    }

    #[test]
    fn test_edge_case_max_u16_offset() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(DataOffset::MAX);
        let mut buffer = Vec::new();
        
        entry_offset.write_to(&mut buffer);
        
        assert_eq!(buffer.len(), 4);
        assert_eq!(DataOffset::from_be_bytes([buffer[0], buffer[1]]), DataOffset::MAX);
        assert_eq!(DataOffset::from_be_bytes([buffer[2], buffer[3]]), 1);
    }

    #[test]
    fn test_edge_case_zero_offset() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(0);
        let mut buffer = Vec::new();
        
        entry_offset.write_to(&mut buffer);
        
        assert_eq!(buffer.len(), 4);
        assert_eq!(DataOffset::from_be_bytes([buffer[0], buffer[1]]), 0);
        assert_eq!(DataOffset::from_be_bytes([buffer[2], buffer[3]]), 1);
    }

    #[test]
    fn test_mixed_operations() {
        // Test combining initial values with added values
        let initial = vec![10, 20];
        let mut entry_offset = MetaEntryOffset::new(Some(initial));
        entry_offset.add(30);
        entry_offset.add(40);
        
        assert_eq!(entry_offset.len(), 10); // 2 + 4*2
        
        let mut buffer = Vec::new();
        entry_offset.write_to(&mut buffer);
        
        assert_eq!(buffer.len(), 10);
        assert_eq!(DataOffset::from_be_bytes([buffer[0], buffer[1]]), 10);
        assert_eq!(DataOffset::from_be_bytes([buffer[2], buffer[3]]), 20);
        assert_eq!(DataOffset::from_be_bytes([buffer[4], buffer[5]]), 30);
        assert_eq!(DataOffset::from_be_bytes([buffer[6], buffer[7]]), 40);
        assert_eq!(DataOffset::from_be_bytes([buffer[8], buffer[9]]), 4); // count
    }
}