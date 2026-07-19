use std::io::{self, Read, Write};

pub fn write_u16<W: Write>(writer: &mut W, value: u16) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

pub fn read_u16<R: Read>(reader: &mut R) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub fn write_i64<W: Write>(writer: &mut W, value: i64) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

pub fn read_i64<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn u16_round_trip() {
        for v in [0u16, 1, 255, 256, 25565, u16::MAX] {
            let mut buf = Vec::new();
            write_u16(&mut buf, v).unwrap();
            let mut cursor = Cursor::new(buf);
            assert_eq!(v, read_u16(&mut cursor).unwrap());
        }
    }

    #[test]
    fn i64_round_trip() {
        for v in [0i64, 1, -1, i64::MAX, i64::MIN, 1234567890] {
            let mut buf = Vec::new();
            write_i64(&mut buf, v).unwrap();
            let mut cursor = Cursor::new(buf);
            assert_eq!(v, read_i64(&mut cursor).unwrap());
        }
    }
}
