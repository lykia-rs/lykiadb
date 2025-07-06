use bytes::BufMut;

use crate::SIZEOF_U16;

pub struct MetaOffsetReg {
    offsets: Vec<u16>,
}

impl MetaOffsetReg {
    pub fn new(initial: Option<Vec<u16>>) -> Self {
        MetaOffsetReg {
            offsets: initial.unwrap_or_else(|| vec![]),
        }
    }

    pub fn add(&mut self, offset: u16) {
        self.offsets.push(offset);
    }

    pub fn finish(&self, buffer: &mut Vec<u8>) {
        buffer.put_u16(self.offsets.len() as u16);
        for &offset in &self.offsets {
            buffer.put_u16(offset);
        }
    }

    pub fn size(&self) -> usize {
        SIZEOF_U16 + self.offsets.len() * SIZEOF_U16 // 2 bytes for count + 2 bytes for each offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_none() {
        let offset_reg = MetaOffsetReg::new(None);
        assert_eq!(offset_reg.offsets.len(), 0);
        assert_eq!(offset_reg.size(), 2); // Just the count field
    }

    #[test]
    fn test_new_with_initial_values() {
        let initial = vec![10, 20, 30];
        let offset_reg = MetaOffsetReg::new(Some(initial.clone()));
        assert_eq!(offset_reg.offsets, initial);
        assert_eq!(offset_reg.size(), 2 + 3 * 2); // count + 3 offsets
    }

    #[test]
    fn test_add_single_offset() {
        let mut offset_reg = MetaOffsetReg::new(None);
        offset_reg.add(100);
        assert_eq!(offset_reg.offsets.len(), 1);
        assert_eq!(offset_reg.offsets[0], 100);
        assert_eq!(offset_reg.size(), 4); // count + 1 offset
    }

    #[test]
    fn test_add_multiple_offsets() {
        let mut offset_reg = MetaOffsetReg::new(None);
        offset_reg.add(100);
        offset_reg.add(200);
        offset_reg.add(300);
        
        assert_eq!(offset_reg.offsets.len(), 3);
        assert_eq!(offset_reg.offsets, vec![100, 200, 300]);
        assert_eq!(offset_reg.size(), 8); // count + 3 offsets
    }

    #[test]
    fn test_size_calculation() {
        let mut offset_reg = MetaOffsetReg::new(None);
        
        // Empty: 2 bytes for count
        assert_eq!(offset_reg.size(), 2);
        
        // One offset: 2 bytes for count + 2 bytes for offset
        offset_reg.add(50);
        assert_eq!(offset_reg.size(), 4);
        
        // Two offsets: 2 bytes for count + 4 bytes for offsets
        offset_reg.add(75);
        assert_eq!(offset_reg.size(), 6);
        
        // Ten offsets: 2 bytes for count + 20 bytes for offsets
        for i in 0..8 {
            offset_reg.add(i * 10);
        }
        assert_eq!(offset_reg.size(), 22);
    }

    #[test]
    fn test_finish_empty() {
        let offset_reg = MetaOffsetReg::new(None);
        let mut buffer = Vec::new();
        
        offset_reg.finish(&mut buffer);
        
        assert_eq!(buffer.len(), 2);
        // Should contain count of 0
        assert_eq!(u16::from_be_bytes([buffer[0], buffer[1]]), 0);
    }

    #[test]
    fn test_finish_single_offset() {
        let mut offset_reg = MetaOffsetReg::new(None);
        offset_reg.add(1024);
        let mut buffer = Vec::new();
        
        offset_reg.finish(&mut buffer);
        
        assert_eq!(buffer.len(), 4);
        // Count should be 1
        assert_eq!(u16::from_be_bytes([buffer[0], buffer[1]]), 1);
        // Offset should be 1024
        assert_eq!(u16::from_be_bytes([buffer[2], buffer[3]]), 1024);
    }

    #[test]
    fn test_finish_multiple_offsets() {
        let mut offset_reg = MetaOffsetReg::new(None);
        offset_reg.add(256);
        offset_reg.add(512);
        offset_reg.add(1024);
        let mut buffer = Vec::new();
        
        offset_reg.finish(&mut buffer);
        
        assert_eq!(buffer.len(), 8); // 2 bytes count + 6 bytes offsets
        
        // Count should be 3
        assert_eq!(u16::from_be_bytes([buffer[0], buffer[1]]), 3);
        // First offset: 256
        assert_eq!(u16::from_be_bytes([buffer[2], buffer[3]]), 256);
        // Second offset: 512
        assert_eq!(u16::from_be_bytes([buffer[4], buffer[5]]), 512);
        // Third offset: 1024
        assert_eq!(u16::from_be_bytes([buffer[6], buffer[7]]), 1024);
    }

    #[test]
    fn test_finish_with_initial_values() {
        let initial = vec![100, 200, 300];
        let offset_reg = MetaOffsetReg::new(Some(initial));
        let mut buffer = Vec::new();
        
        offset_reg.finish(&mut buffer);
        
        assert_eq!(buffer.len(), 8);
        // Count should be 3
        assert_eq!(u16::from_be_bytes([buffer[0], buffer[1]]), 3);
        // Check each offset
        assert_eq!(u16::from_be_bytes([buffer[2], buffer[3]]), 100);
        assert_eq!(u16::from_be_bytes([buffer[4], buffer[5]]), 200);
        assert_eq!(u16::from_be_bytes([buffer[6], buffer[7]]), 300);
    }

    #[test]
    fn test_finish_appends_to_existing_buffer() {
        let mut offset_reg = MetaOffsetReg::new(None);
        offset_reg.add(42);
        
        let mut buffer = vec![0xFF, 0xFE]; // Pre-existing data
        offset_reg.finish(&mut buffer);
        
        assert_eq!(buffer.len(), 6); // 2 existing + 2 count + 2 offset
        // Pre-existing data should remain
        assert_eq!(buffer[0], 0xFF);
        assert_eq!(buffer[1], 0xFE);
        // New data appended
        assert_eq!(u16::from_be_bytes([buffer[2], buffer[3]]), 1);
        assert_eq!(u16::from_be_bytes([buffer[4], buffer[5]]), 42);
    }

    #[test]
    fn test_edge_case_max_u16_offset() {
        let mut offset_reg = MetaOffsetReg::new(None);
        offset_reg.add(u16::MAX);
        let mut buffer = Vec::new();
        
        offset_reg.finish(&mut buffer);
        
        assert_eq!(buffer.len(), 4);
        assert_eq!(u16::from_be_bytes([buffer[0], buffer[1]]), 1);
        assert_eq!(u16::from_be_bytes([buffer[2], buffer[3]]), u16::MAX);
    }

    #[test]
    fn test_edge_case_zero_offset() {
        let mut offset_reg = MetaOffsetReg::new(None);
        offset_reg.add(0);
        let mut buffer = Vec::new();
        
        offset_reg.finish(&mut buffer);
        
        assert_eq!(buffer.len(), 4);
        assert_eq!(u16::from_be_bytes([buffer[0], buffer[1]]), 1);
        assert_eq!(u16::from_be_bytes([buffer[2], buffer[3]]), 0);
    }

    #[test]
    fn test_mixed_operations() {
        // Test combining initial values with added values
        let initial = vec![10, 20];
        let mut offset_reg = MetaOffsetReg::new(Some(initial));
        offset_reg.add(30);
        offset_reg.add(40);
        
        assert_eq!(offset_reg.size(), 10); // 2 + 4*2
        
        let mut buffer = Vec::new();
        offset_reg.finish(&mut buffer);
        
        assert_eq!(buffer.len(), 10);
        assert_eq!(u16::from_be_bytes([buffer[0], buffer[1]]), 4); // count
        assert_eq!(u16::from_be_bytes([buffer[2], buffer[3]]), 10);
        assert_eq!(u16::from_be_bytes([buffer[4], buffer[5]]), 20);
        assert_eq!(u16::from_be_bytes([buffer[6], buffer[7]]), 30);
        assert_eq!(u16::from_be_bytes([buffer[8], buffer[9]]), 40);
    }
}