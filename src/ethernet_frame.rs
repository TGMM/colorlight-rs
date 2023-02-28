use core::{mem::size_of, slice::from_raw_parts};

#[repr(packed)]
pub struct EthernetFrame<const DATA_LEN: usize> {
    pub dst_mac_addr: [u8; 6],
    pub src_mac_addr: [u8; 6],
    pub ether_type: u16,
    pub data: [u8; DATA_LEN],
}

impl<const DATA_LEN: usize> EthernetFrame<DATA_LEN> {
    pub unsafe fn as_u8_slice(&self) -> &[u8] {
        from_raw_parts((self as *const Self) as *const u8, size_of::<Self>())
    }
}

#[cfg(test)]
mod test {
    use super::EthernetFrame;

    #[test]
    fn frame_as_u8() {
        let frame = EthernetFrame {
            dst_mac_addr: [0x00, 0x01, 0x02, 0x03, 0x05, 0x06],
            src_mac_addr: [0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C],
            ether_type: 0x0107,
            data: [0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA],
        };

        const MAC_ADDR_SIZE: usize = 6;
        const DATA_LEN: usize = 6;
        const ETHER_TYPE_SIZE: usize = 2;
        const SLICE_SIZE: usize = (MAC_ADDR_SIZE * 2) + DATA_LEN + ETHER_TYPE_SIZE;

        #[rustfmt::skip]
        const FRAME_ARR: [u8; SLICE_SIZE] = [
            0x00, 0x01, 0x02, 0x03, 0x05, 0x06, 
            0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 
            // U16 is least-significant first, so we invert them here
            0x07, 0x01, 
            0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA
        ];

        unsafe { assert_eq!(frame.as_u8_slice().len(), SLICE_SIZE) }
        unsafe { assert_eq!(frame.as_u8_slice(), &FRAME_ARR) }
    }
}
