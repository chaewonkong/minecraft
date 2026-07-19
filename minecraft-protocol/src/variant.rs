use std::io::{self, Read, Write};

#[derive(Debug)]
pub enum VarIntError {
    Io(io::Error),
    TooLong,
}

impl From<io::Error> for VarIntError {
    fn from(e: io::Error) -> Self {
        VarIntError::Io(e)
    }
}

pub fn write_varint<W: Write>(writer: &mut W, value: i32) -> Result<(), VarIntError> {
    let mut val = value as u32;
    loop {
        if val & !0x7F == 0 {
            // 0x7F = 0000...0111 1111 하위 7비트만 1. not(!)으로 하위 7비트만 0으로 변경
            // 남은  비트가 하위 7비트 안에 모두 포함된다.
            writer.write_all(&[val as u8])?;
            return Ok(());
        }
        writer.write_all(&[(val as u8 & 0x7F) | 0x80])?; // 0x80 = 1000 0000 첫비트만 1로 킨다.
        val >>= 7; // 7비트 시프트
    }
}

pub fn read_varint<R: Read>(reader: &mut R) -> Result<i32, VarIntError> {
    let mut result: i32 = 0;
    let mut shift = 0;

    loop {
        let mut buf = [0u8; 1]; // u8 타입의 0 하나가 포함된 길이 "1" 짜리 배열: [초기값; 길이]
        reader.read_exact(&mut buf)?; // stream에서 한 byte씩 읽기
        let byte = buf[0]; // 첫번째 값 꺼내기

        // byte에서 하위 7비트를 읽고 i32로 바꾼뒤 shift -> result 할당
        // 매번 7비트씩 shift하고 |로 result에 추가하므로 사실상 덧셈과 같음.
        result |= ((byte & 0x7F) as i32) << shift;
        if byte & 0x80 == 0 {
            // continuation bit가 0, 즉 7비트로 끝
            return Ok(result);
        }

        shift += 7; // result에 다음 값을 더하기 위해 7비트 시프트
        if shift >= 32 {
            // VarInt는 최대 5바이트.
            return Err(VarIntError::TooLong);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn round_trip(value: i32) {
        let mut buf = Vec::new();
        write_varint(&mut buf, value).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = read_varint(&mut cursor).unwrap();
        assert_eq!(value, decoded, "round-trip failed for {value}");
    }

    #[test]
    fn varint_boundary_values() {
        for v in [0, 127, 128, 300, i32::MAX, -1, i32::MIN] {
            round_trip(v);
        }
    }
}
