use std::io::{self, Read};
use std::slice;

pub fn read_little_endian_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_leb128_signed<R: Read>(reader: &mut R, count: usize) -> io::Result<Vec<i32>> {
    read_leb128_signed_checked(reader, count, true)
}

pub fn read_leb128_signed_checked<R: Read>(
    reader: &mut R,
    count: usize,
    check_magic: bool,
) -> io::Result<Vec<i32>> {
    let compressed_data = read_leb128_payload(reader, check_magic)?;
    let total_bytes = compressed_data.len();

    let mut result = Vec::with_capacity(count);
    let mut buf_pos = 0;

    for _ in 0..count {
        let mut value: i32 = 0;
        let mut shift = 0;
        loop {
            if buf_pos >= total_bytes {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF in LEB128 stream",
                ));
            }
            let byte = compressed_data[buf_pos];
            buf_pos += 1;

            value |= ((byte & 0x7f) as i32) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                if shift < 32 && (byte & 0x40) != 0 {
                    value |= !((1 << shift) - 1);
                }
                result.push(value);
                break;
            }
        }
    }

    Ok(result)
}

pub fn read_leb128_i16<R: Read>(reader: &mut R, count: usize) -> io::Result<Vec<i16>> {
    read_leb128_i16_checked(reader, count, true)
}

pub fn read_leb128_i16_checked<R: Read>(
    reader: &mut R,
    count: usize,
    check_magic: bool,
) -> io::Result<Vec<i16>> {
    let compressed_data = read_leb128_payload(reader, check_magic)?;
    let total_bytes = compressed_data.len();

    let mut result = Vec::with_capacity(count);
    let mut buf_pos = 0;

    for _ in 0..count {
        let mut value: i32 = 0;
        let mut shift = 0;
        loop {
            if buf_pos >= total_bytes {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF in LEB128 stream",
                ));
            }

            let byte = compressed_data[buf_pos];
            buf_pos += 1;

            value |= ((byte & 0x7f) as i32) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                if shift < 32 && (byte & 0x40) != 0 {
                    value |= !((1 << shift) - 1);
                }
                result.push(value as i16);
                break;
            }
        }
    }

    Ok(result)
}

pub fn read_leb128_i32<R: Read>(reader: &mut R, count: usize) -> io::Result<Vec<i32>> {
    read_leb128_signed(reader, count)
}

pub fn read_i32_array<R: Read>(reader: &mut R, count: usize) -> io::Result<Vec<i32>> {
    let mut result = Vec::with_capacity(count);
    let mut buf = [0u8; 4];
    for _ in 0..count {
        reader.read_exact(&mut buf)?;
        result.push(i32::from_le_bytes(buf));
    }
    Ok(result)
}

pub fn read_i8_array<R: Read>(reader: &mut R, count: usize) -> io::Result<Vec<i8>> {
    let mut result = vec![0i8; count];
    let bytes = unsafe { slice::from_raw_parts_mut(result.as_mut_ptr() as *mut u8, count) };
    reader.read_exact(bytes)?;
    Ok(result)
}

fn read_leb128_payload<R: Read>(reader: &mut R, check_magic: bool) -> io::Result<Vec<u8>> {
    if check_magic {
        let mut magic = [0u8; 17];
        reader.read_exact(&mut magic)?;
        if &magic != b"COMPRESSED_LEB128" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid LEB128 magic string: {:?}", magic),
            ));
        }
    }

    let total_bytes = read_little_endian_u32(reader)? as usize;
    let mut compressed_data = vec![0u8; total_bytes];
    reader.read_exact(&mut compressed_data)?;
    Ok(compressed_data)
}
