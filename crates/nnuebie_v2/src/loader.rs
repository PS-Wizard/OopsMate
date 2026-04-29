use std::io::{self, Read};

const LEB128_MAGIC: &[u8; 17] = b"COMPRESSED_LEB128";
const LEB128_BUFFER_SIZE: usize = 4096;

#[inline(always)]
pub fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

#[inline(always)]
pub fn read_i32_array<R: Read>(reader: &mut R, count: usize) -> io::Result<Box<[i32]>> {
    let mut values = vec![0i32; count];
    for value in &mut values {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        *value = i32::from_le_bytes(bytes);
    }
    Ok(values.into_boxed_slice())
}

#[inline(always)]
pub fn read_i8_array<R: Read>(reader: &mut R, count: usize) -> io::Result<Box<[i8]>> {
    let mut values = vec![0i8; count];
    let bytes =
        unsafe { std::slice::from_raw_parts_mut(values.as_mut_ptr().cast::<u8>(), values.len()) };
    reader.read_exact(bytes)?;
    Ok(values.into_boxed_slice())
}

pub fn read_leb128_i16_array<R: Read>(reader: &mut R, count: usize) -> io::Result<Box<[i16]>> {
    let mut values = vec![0i16; count];
    read_leb128_into_i16(reader, &mut values)?;
    Ok(values.into_boxed_slice())
}

pub fn read_leb128_i32_array<R: Read>(reader: &mut R, count: usize) -> io::Result<Box<[i32]>> {
    let mut values = vec![0i32; count];
    read_leb128_into_i32(reader, &mut values)?;
    Ok(values.into_boxed_slice())
}

fn read_leb128_header<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut magic = [0u8; 17];
    reader.read_exact(&mut magic)?;
    if &magic != LEB128_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid LEB128 magic",
        ));
    }

    read_u32(reader)
}

fn read_leb128_into_i16<R: Read>(reader: &mut R, output: &mut [i16]) -> io::Result<()> {
    let mut bytes_left = read_leb128_header(reader)? as usize;
    let mut buffer = [0u8; LEB128_BUFFER_SIZE];
    let mut buf_pos = 0usize;
    let mut buf_len = 0usize;

    for value in output {
        let mut result = 0i32;
        let mut shift = 0u32;

        loop {
            if buf_pos >= buf_len {
                if bytes_left == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "unexpected EOF in i16 LEB128 stream",
                    ));
                }

                let chunk = bytes_left.min(LEB128_BUFFER_SIZE);
                reader.read_exact(&mut buffer[..chunk])?;
                buf_pos = 0;
                buf_len = chunk;
            }

            let byte = buffer[buf_pos];
            buf_pos += 1;
            bytes_left -= 1;

            result |= i32::from(byte & 0x7f) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                if shift < 32 && (byte & 0x40) != 0 {
                    result |= !((1i32 << shift) - 1);
                }
                *value = result as i16;
                break;
            }
        }
    }

    if bytes_left != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "extra bytes left in i16 LEB128 stream",
        ));
    }

    Ok(())
}

fn read_leb128_into_i32<R: Read>(reader: &mut R, output: &mut [i32]) -> io::Result<()> {
    let mut bytes_left = read_leb128_header(reader)? as usize;
    let mut buffer = [0u8; LEB128_BUFFER_SIZE];
    let mut buf_pos = 0usize;
    let mut buf_len = 0usize;

    for value in output {
        let mut result = 0i32;
        let mut shift = 0u32;

        loop {
            if buf_pos >= buf_len {
                if bytes_left == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "unexpected EOF in i32 LEB128 stream",
                    ));
                }

                let chunk = bytes_left.min(LEB128_BUFFER_SIZE);
                reader.read_exact(&mut buffer[..chunk])?;
                buf_pos = 0;
                buf_len = chunk;
            }

            let byte = buffer[buf_pos];
            buf_pos += 1;
            bytes_left -= 1;

            result |= i32::from(byte & 0x7f) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                if shift < 32 && (byte & 0x40) != 0 {
                    result |= !((1i32 << shift) - 1);
                }
                *value = result;
                break;
            }
        }
    }

    if bytes_left != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "extra bytes left in i32 LEB128 stream",
        ));
    }

    Ok(())
}
