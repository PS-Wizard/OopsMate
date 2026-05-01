use std::io::Write;

pub fn write_tables<W: Write>(
    mut out: W,
    pawn_attacks: &[[u64; 64]; 2],
    knight_attacks: &[u64; 64],
    king_attacks: &[u64; 64],
    rook_masks: &[u64; 64],
    bishop_masks: &[u64; 64],
    rook_offsets: &[u32; 64],
    bishop_offsets: &[u32; 64],
    rook_attacks: &[u64],
    bishop_attacks: &[u64],
    between: &[[u64; 64]; 64],
    through: &[[u64; 64]; 64],
) -> std::io::Result<()> {
    writeln!(out, "pub const WHITE: usize = 0;")?;
    writeln!(out, "pub const BLACK: usize = 1;")?;
    write_u64_matrix(&mut out, "PAWN_ATTACKS", pawn_attacks)?;
    write_u64_array(&mut out, "KNIGHT_ATTACKS", knight_attacks)?;
    write_u64_array(&mut out, "KING_ATTACKS", king_attacks)?;
    write_u64_array(&mut out, "ROOK_MASKS", rook_masks)?;
    write_u64_array(&mut out, "BISHOP_MASKS", bishop_masks)?;
    write_u32_array(&mut out, "ROOK_OFFSETS", rook_offsets)?;
    write_u32_array(&mut out, "BISHOP_OFFSETS", bishop_offsets)?;
    write_u64_flat(&mut out, "ROOK_ATTACKS", rook_attacks)?;
    write_u64_flat(&mut out, "BISHOP_ATTACKS", bishop_attacks)?;
    write_u64_grid(&mut out, "BETWEEN", between)?;
    write_u64_grid(&mut out, "THROUGH", through)?;
    Ok(())
}

fn write_u64_array<W: Write>(out: &mut W, name: &str, values: &[u64; 64]) -> std::io::Result<()> {
    writeln!(out, "pub const {name}: [u64; 64] = [")?;
    for &value in values {
        writeln!(out, "    0x{value:016x},")?;
    }
    writeln!(out, "];\n")
}

fn write_u32_array<W: Write>(out: &mut W, name: &str, values: &[u32; 64]) -> std::io::Result<()> {
    writeln!(out, "pub const {name}: [u32; 64] = [")?;
    for &value in values {
        writeln!(out, "    {value},")?;
    }
    writeln!(out, "];\n")
}

fn write_u64_flat<W: Write>(out: &mut W, name: &str, values: &[u64]) -> std::io::Result<()> {
    writeln!(out, "pub const {name}: [u64; {}] = [", values.len())?;
    for &value in values {
        writeln!(out, "    0x{value:016x},")?;
    }
    writeln!(out, "];\n")
}

fn write_u64_matrix<W: Write>(
    out: &mut W,
    name: &str,
    values: &[[u64; 64]; 2],
) -> std::io::Result<()> {
    writeln!(out, "pub const {name}: [[u64; 64]; 2] = [")?;
    for row in values {
        writeln!(out, "    [")?;
        for &value in row {
            writeln!(out, "        0x{value:016x},")?;
        }
        writeln!(out, "    ],")?;
    }
    writeln!(out, "];\n")
}

fn write_u64_grid<W: Write>(
    out: &mut W,
    name: &str,
    values: &[[u64; 64]; 64],
) -> std::io::Result<()> {
    writeln!(out, "pub const {name}: [[u64; 64]; 64] = [")?;
    for row in values {
        writeln!(out, "    [")?;
        for &value in row {
            writeln!(out, "        0x{value:016x},")?;
        }
        writeln!(out, "    ],")?;
    }
    writeln!(out, "];\n")
}
