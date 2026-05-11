use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// UDP 音频包结构 (小端序)
///
/// [0..4)   SequenceID  u32   包序号
/// [4..12)  Timestamp   u64   采集时间戳微秒
/// [12..14) PayloadLen  u16   Opus 数据长度
/// [14..]   Opus Data   &[u8] Opus 编码音频数据
#[derive(Debug, Clone)]
pub struct AudioPacket {
    pub sequence: u32,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}

/// 解析 UDP 数据包
pub fn parse_packet(data: &[u8]) -> Option<AudioPacket> {
    if data.len() < 14 {
        log::warn!("Packet too short: {} bytes", data.len());
        return None;
    }

    let mut cursor = Cursor::new(data);

    let sequence = cursor.read_u32::<LittleEndian>().ok()?;
    let timestamp = cursor.read_u64::<LittleEndian>().ok()?;
    let payload_len = cursor.read_u16::<LittleEndian>().ok()? as usize;

    if data.len() < 14 + payload_len {
        log::warn!(
            "Packet payload truncated: header=14, declared={}, actual={}",
            payload_len,
            data.len() - 14
        );
        return None;
    }

    let mut payload = vec![0u8; payload_len];
    if payload_len > 0 {
        cursor.read_exact(&mut payload).ok()?;
    }

    Some(AudioPacket {
        sequence,
        timestamp,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_packet() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u32.to_le_bytes());      // sequence
        buf.extend_from_slice(&12345678u64.to_le_bytes()); // timestamp
        buf.extend_from_slice(&4u16.to_le_bytes());       // payload_len
        buf.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // payload

        let packet = parse_packet(&buf).unwrap();
        assert_eq!(packet.sequence, 1);
        assert_eq!(packet.timestamp, 12345678);
        assert_eq!(packet.payload, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_parse_too_short() {
        assert!(parse_packet(&[0u8; 10]).is_none());
    }

    #[test]
    fn test_parse_truncated_payload() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(&0u64.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes()); // claims 100 bytes but none follow
        buf.push(0x01);
        assert!(parse_packet(&buf).is_none());
    }
}
