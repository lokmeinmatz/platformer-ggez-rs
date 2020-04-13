use async_std::prelude::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum PacketType {
    EntitiesFrameData = 0x01,
    ChunkData = 0x02
}

impl TryFrom<u8> for PacketType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            x if x == PacketType::EntitiesFrameData as u8 => PacketType::EntitiesFrameData,
            x if x == PacketType::ChunkData as u8 => PacketType::ChunkData,
            _ => return Err(())
        })
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct EntityNetworkData {
    id: u64,
    pos: (f32, f32),
    vel: (f32, f32)
}


#[derive(Debug)]
pub enum Packet {
    EntitiesFrameData(Box<[EntityNetworkData]>)

}

const MAX_PAYLOAD_SIZE: usize = 4;

use async_std::net;
use std::convert::TryFrom;
use std::mem::MaybeUninit;

/// Decodes the next network packet
/// Structure:
/// 0x0    0x1    0x4          0x8     0x8 + packet_len
///  +------+------+------------+- ... ---+
///  | type | 'PKG'| packet_len | payload |
///
/// EntitiesFrameData payload:
///  +--------------+--------- ... ----------------+
///  | 32b enti_cnt | enti_cnt * EntityNetworkData |
pub async fn decode_next_packet(header_buf: &[u8; 8], stream: &mut net::TcpStream) -> Result<Packet, &'static str> {

    let ptype = PacketType::try_from(header_buf[0]).map_err(|_| "unknown packet type")?;

    if &header_buf[1..4] != b"PKG" {
        return Err("Packet signature invalid");
    }


    let payload_len = u32::from_be_bytes(
        [header_buf[4], header_buf[5], header_buf[6], header_buf[7]]) as usize;

    assert!(header_buf.len() >= payload_len);

    match ptype {
        PacketType::EntitiesFrameData => {
            // get how long the entity list is
            let entities_to_sync = {
                let mut buf = [0u8; 4];
                stream.read_exact(&mut buf).await.map_err(|_| "Failed to read len of entites to sync")?;
                u32::from_be_bytes(buf)
            };

            // allocate empty buffer
            let mut network_e_data: Box<[MaybeUninit<EntityNetworkData>]> = Box::new_uninit_slice(entities_to_sync as usize);

            unsafe {
                let net_buffer = std::slice::from_raw_parts_mut(network_e_data.as_mut_ptr() as *mut u8, std::mem::size_of_val(network_e_data.as_ref()));
                stream.read_exact(net_buffer).await.unwrap();
                return Ok(Packet::EntitiesFrameData(network_e_data.assume_init()))
            }
        }
        _ => unimplemented!()
    }
}