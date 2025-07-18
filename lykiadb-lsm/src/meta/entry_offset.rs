use bytes::BufMut;

use crate::block::builder::{DataOffsetLen, SIZEOF_DATA_OFFSET_LEN};

pub(crate) struct MetaEntryOffset {
    offsets: Vec<DataOffsetLen>,
}

impl MetaEntryOffset {
    pub fn new(initial: Option<Vec<DataOffsetLen>>) -> Self {
        MetaEntryOffset {
            offsets: initial.unwrap_or_default(),
        }
    }

    pub fn add(&mut self, offset: DataOffsetLen) {
        self.offsets.push(offset);
    }

    // ------------------------------------------------------------------
    // |   offset (u32)   | ... |    offset (u32)   |    length (u32)   |
    // ------------------------------------------------------------------
    pub fn write_to(&self, buffer: &mut Vec<u8>) {
        for &offset in &self.offsets {
            buffer.put_u32(offset);
        }
        buffer.put_u32(self.offsets.len() as DataOffsetLen);
    }

    pub fn len(&self) -> usize {
        SIZEOF_DATA_OFFSET_LEN + self.offsets.len() * SIZEOF_DATA_OFFSET_LEN // 4 bytes for count + 4 bytes for each offset
    }

    pub fn from_buffer(buffer: &[u8]) -> Vec<DataOffsetLen> {
        buffer
            .chunks_exact(SIZEOF_DATA_OFFSET_LEN)
            .map(|chunk| DataOffsetLen::from_be_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<_>>()
    }

    pub fn to_vec(&self) -> Vec<u32> {
        self.offsets.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_none() {
        let entry_offset = MetaEntryOffset::new(None);
        assert_eq!(entry_offset.offsets.len(), 0);
        assert_eq!(entry_offset.len(), 4); // Just the count field
    }

    #[test]
    fn test_new_with_initial_values() {
        let initial = vec![10, 20, 30];
        let entry_offset = MetaEntryOffset::new(Some(initial.clone()));
        assert_eq!(entry_offset.offsets, initial);
        assert_eq!(entry_offset.len(), 4 + 3 * 4); // count + 3 offsets
    }

    #[test]
    fn test_add_single_offset() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(100);
        assert_eq!(entry_offset.offsets.len(), 1);
        assert_eq!(entry_offset.offsets[0], 100);
        assert_eq!(entry_offset.len(), 8); // count + 1 offset
    }

    #[test]
    fn test_add_multiple_offsets() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(100);
        entry_offset.add(200);
        entry_offset.add(300);

        assert_eq!(entry_offset.offsets.len(), 3);
        assert_eq!(entry_offset.offsets, vec![100, 200, 300]);
        assert_eq!(entry_offset.len(), 16); // count + 3 offsets
    }

    #[test]
    fn test_size_calculation() {
        let mut entry_offset = MetaEntryOffset::new(None);

        // Empty: 4 bytes for count
        assert_eq!(entry_offset.len(), 4);

        // One offset: 4 bytes for count + 4 bytes for offset
        entry_offset.add(50);
        assert_eq!(entry_offset.len(), 8);

        // Two offsets: 4 bytes for count + 8 bytes for offsets
        entry_offset.add(75);
        assert_eq!(entry_offset.len(), 12);

        // Ten offsets: 4 bytes for count + 40 bytes for offsets
        for i in 0..8 {
            entry_offset.add(i * 10);
        }
        assert_eq!(entry_offset.len(), 44);
    }

    #[test]
    fn test_finish_empty() {
        let entry_offset = MetaEntryOffset::new(None);
        let mut buffer = Vec::new();

        entry_offset.write_to(&mut buffer);

        assert_eq!(buffer.len(), 4);
        // Should contain count of 0
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            0
        );
    }

    #[test]
    fn test_finish_single_offset() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(1024);
        let mut buffer = Vec::new();

        entry_offset.write_to(&mut buffer);

        assert_eq!(buffer.len(), 8); // 4 bytes for offset + 4 bytes for count
        // Offset should be 1024
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            1024
        );
        // Count should be 1
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            1
        );
    }

    #[test]
    fn test_finish_multiple_offsets() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(256);
        entry_offset.add(512);
        entry_offset.add(1024);
        let mut buffer = Vec::new();

        entry_offset.write_to(&mut buffer);

        assert_eq!(buffer.len(), 16); // 3 * 4 bytes offsets + 4 bytes count

        // First offset: 256
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            256
        );
        // Second offset: 512
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            512
        );
        // Third offset: 1024
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
            1024
        );
        // Count should be 3
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
            3
        );
    }

    #[test]
    fn test_finish_with_initial_values() {
        let initial = vec![100, 200, 300];
        let entry_offset = MetaEntryOffset::new(Some(initial));
        let mut buffer = Vec::new();

        entry_offset.write_to(&mut buffer);

        assert_eq!(buffer.len(), 16); // 4 + 4 * 3
        // Check each offset
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            100
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            200
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
            300
        );
        // Count should be 3
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
            3
        );
    }

    #[test]
    fn test_finish_appends_to_existing_buffer() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(42);

        let mut buffer = vec![0xFF, 0xFE]; // Pre-existing data
        entry_offset.write_to(&mut buffer);

        assert_eq!(buffer.len(), 10); // 2 existing + 4 offset + 4 count
        // Pre-existing data should remain
        assert_eq!(buffer[0], 0xFF);
        assert_eq!(buffer[1], 0xFE);
        // New data appended
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[2], buffer[3], buffer[4], buffer[5]]),
            42
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[6], buffer[7], buffer[8], buffer[9]]),
            1
        );
    }

    #[test]
    fn test_edge_case_max_u16_offset() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(DataOffsetLen::MAX);
        let mut buffer = Vec::new();

        entry_offset.write_to(&mut buffer);

        assert_eq!(buffer.len(), 8); // 4 + 4
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            DataOffsetLen::MAX
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            1
        );
    }

    #[test]
    fn test_edge_case_zero_offset() {
        let mut entry_offset = MetaEntryOffset::new(None);
        entry_offset.add(0);
        let mut buffer = Vec::new();

        entry_offset.write_to(&mut buffer);

        assert_eq!(buffer.len(), 8); // 4 + 4
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            0
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            1
        );
    }

    #[test]
    fn test_mixed_operations() {
        // Test combining initial values with added values
        let initial = vec![10, 20];
        let mut entry_offset = MetaEntryOffset::new(Some(initial));
        entry_offset.add(30);
        entry_offset.add(40);

        assert_eq!(entry_offset.len(), 20); // 4 + 4 * 4

        let mut buffer = Vec::new();
        entry_offset.write_to(&mut buffer);

        assert_eq!(buffer.len(), 20);
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            10
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            20
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
            30
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
            40
        );
        assert_eq!(
            DataOffsetLen::from_be_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]),
            4
        ); // count
    }
}
