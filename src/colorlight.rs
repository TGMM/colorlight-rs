use crate::ethernet_frame::EthernetFrame;
use core::intrinsics::roundf32;

const DEFAULT_DST_MAC: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
const DEFAULT_SRC_MAC: [u8; 6] = [0x22, 0x22, 0x33, 0x44, 0x55, 0x66];

const DISPLAY_PACKET_DATA_SIZE: usize = 98;
pub fn create_display_packet(brightness_percentage: u8) -> EthernetFrame<DISPLAY_PACKET_DATA_SIZE> {
    const DISPLAY_ETHER_TYPE: u16 = 0x0107; // Have also seen 0x0100, 0x0104, 0x0107

    let mut packet: EthernetFrame<DISPLAY_PACKET_DATA_SIZE> = EthernetFrame {
        dst_mac_addr: DEFAULT_DST_MAC,
        src_mac_addr: DEFAULT_SRC_MAC,
        ether_type: DISPLAY_ETHER_TYPE.swap_bytes(),
        data: [0x00; DISPLAY_PACKET_DATA_SIZE],
    };

    // Brightness
    //  0x00: 0% brightness
    //  0x03: 1%, 0x05: 2%, 0x08: 3%, 0x0a: 4%, 0x0d: 5%, 0x0f: 6%, 0x1a: 10%
    //  0x40: 25% brightness
    //  0x80: 50% brightness
    //  0xbf: 75% brightness
    //  0xff: 100% brightness
    let brightness_byte = brightness_percent_to_byte(brightness_percentage);
    packet.data[11] = brightness_byte;

    // No idea lol
    packet.data[12] = 0x05;

    // Brightness (Color temperature)
    //  2000K at 10% brightness: 0x1a, 0x0c, 0x01
    //  6500K at 10% brightness: 0x1a, 0x1a, 0x1a
    //  2000K at 100% brightness: 0xff, 0x76, 0x06
    //  4500K at 100% brightness: 0xff, 0xdc, 0x8f
    //  6500K at 100% brightness: 0xff, 0xff, 0xff
    //  8000K at 100% brightness: 0xce, 0xd8, 0xff
    packet.data[14] = brightness_byte;
    packet.data[15] = brightness_byte;
    packet.data[16] = brightness_byte;

    packet
}

const BRIGHTNESS_PACKET_DATA_SIZE: usize = 63;
pub fn create_brightness_packet(
    brightness_percentage: u8,
) -> EthernetFrame<BRIGHTNESS_PACKET_DATA_SIZE> {
    let brightness_byte = brightness_percent_to_byte(brightness_percentage);

    let mut packet: EthernetFrame<BRIGHTNESS_PACKET_DATA_SIZE> = EthernetFrame {
        dst_mac_addr: DEFAULT_DST_MAC,                              // 6
        src_mac_addr: DEFAULT_SRC_MAC,                              // 6
        ether_type: (0x0A00 + brightness_byte as u16).swap_bytes(), // 2
        data: [0x00; BRIGHTNESS_PACKET_DATA_SIZE],
    };

    // Brightness
    //  1%=0x28, 2%=0x35, 5%=0x4d, 25%=0x92 , 50%=0xc1,
    //  75%=0xe3, 100%=0xff (based on LEDVISION software)
    packet.data[0] = brightness_byte;
    packet.data[1] = brightness_byte;

    // IDK what it does, but it's always 0xFF
    packet.data[2] = 0xFF;

    packet
}

fn brightness_percent_to_byte(brightness_percentage: u8) -> u8 {
    let clamped_brightness_percent = brightness_percentage.clamp(0, 100);
    let brightness = unsafe { roundf32(clamped_brightness_percent as f32 * 2.55) };
    let clamped_brightness = brightness.clamp(0.0, 255.0);

    clamped_brightness as u8
}

#[inline]
fn get_msb(b: u16) -> u8 {
    b.to_be_bytes()[0]
}

#[inline]
fn get_lsb(b: u16) -> u8 {
    b.to_be_bytes()[1]
}

pub fn create_row_data_packet<const ROW_WIDTH: usize>(
    row_num: u16,
    pixel_offset: u16,
    pixel_count: u16,
    color_data: &[u8; ROW_WIDTH * 3],
) -> EthernetFrame<{ (ROW_WIDTH * 3) + 7 }> {
    assert_eq!(color_data.len(), (ROW_WIDTH * 3));

    let mut packet: EthernetFrame<{ (ROW_WIDTH * 3) + 7 }> = EthernetFrame {
        dst_mac_addr: DEFAULT_DST_MAC,
        src_mac_addr: DEFAULT_SRC_MAC,
        ether_type: (0x5500 + get_msb(row_num) as u16).swap_bytes(),
        data: [0x00; { (ROW_WIDTH * 3) + 7 }],
    };

    packet.data[0] = get_lsb(row_num);
    packet.data[1] = get_msb(pixel_offset);
    packet.data[2] = get_lsb(pixel_offset);
    packet.data[3] = get_msb(pixel_count);
    packet.data[4] = get_lsb(pixel_count);
    packet.data[5] = 0x08;
    packet.data[6] = 0x88;

    let data_ptr = core::ptr::addr_of_mut!(packet.data);
    unsafe {
        let data_ref = data_ptr.as_mut().expect("Error trying to dereference data");
        // Copy color_data into the packet data
        for (dst, src) in data_ref.iter_mut().skip(7).zip(color_data) {
            *dst = *src;
        }
    };

    packet
}

#[cfg(test)]
mod test {
    use super::{
        create_brightness_packet, create_display_packet, create_row_data_packet, get_lsb, get_msb,
    };

    #[test]
    fn get_u16_bytes() {
        const NUM: u16 = 0x0100; // 256

        // Most significant
        assert_eq!(0x01, get_msb(NUM));
        // Least significant
        assert_eq!(0x00, get_lsb(NUM));
    }

    #[test]
    fn brightness_packet() {
        let bp = create_brightness_packet(1);
        // Extracted from the C++ implementation
        const BP_CPP: [u8; 77] = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x22, 0x22, 0x33, 0x44, 0x55, 0x66, 0x0A, 0x03,
            0x03, 0x03, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let bp_slice = unsafe { bp.as_u8_slice() };
        assert_eq!(bp_slice.len(), BP_CPP.len());
        assert_eq!(bp_slice, &BP_CPP);
    }

    #[test]
    fn display_packet() {
        let dp = create_display_packet(1);
        // Extracted from the C++ implementation
        const DP_CPP: [u8; 112] = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x22, 0x22, 0x33, 0x44, 0x55, 0x66, 0x01, 0x07,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x05, 0x00,
            0x03, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let dp_slice = unsafe { dp.as_u8_slice() };
        assert_eq!(dp_slice.len(), DP_CPP.len());
        assert_eq!(dp_slice, &DP_CPP);
    }

    #[test]
    fn row_data_packet() {
        let color_data = [0x00; 9];
        let rdp = create_row_data_packet::<3>(2, 3, 4, &color_data);

        const RDP_CPP: [u8; 30] = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x22, 0x22, 0x33, 0x44, 0x55, 0x66, 0x55, 0x00,
            0x02, 0x00, 0x03, 0x00, 0x04, 0x08, 0x88, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let rdp_slice = unsafe { rdp.as_u8_slice() };
        assert_eq!(rdp_slice.len(), RDP_CPP.len());
        assert_eq!(rdp_slice, &RDP_CPP);
    }
}
