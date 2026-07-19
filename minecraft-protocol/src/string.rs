use crate::varint::{VarIntError, read_varint, write_varint};
use std::io::{Read, Write};

pub fn write_string<W: Write>(writer: &mut W, value: &str) -> Result<(), VarIntError> {
    let bytes = value.as_bytes();
    write_varint(writer, bytes.len() as i32)?; // String의 길이를 VarInt로 전송
    writer.write_all(bytes)?; // 실제 String을 이어서 전송
    Ok(())
}

pub fn read_string<R: Read>(reader: &mut R) -> Result<String, VarIntError> {
    let len = read_varint(reader)?; // continuation bit가 0이 될 떄까지 VarInt로 간주하고 계속 읽음.
    let mut buf = vec![0u8; len as usize]; // VarInt -> i32로 읽을 string의 길이를 확정
    reader.read_exact(&mut buf)?;
    let s = String::from_utf8(buf).map_err(|_| VarIntError::TooLong)?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn round_trip(s: &str) {
        let mut buf = Vec::new();
        write_string(&mut buf, s).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = read_string(&mut cursor).unwrap();
        assert_eq!(s, decoded);
    }

    #[test]
    fn string_round_trip() {
        round_trip("");
        round_trip("hello");
        round_trip("안녕하세요"); // 한글: 글자당 3바이트
        round_trip("Minecraft 25565 🎮"); // 이모지: 4바이트
    }
}
