use crate::aligned::AlignedBuffer;
use crate::loader::{read_leb128_i16, read_leb128_i16_checked, read_leb128_i32};
use std::io::{self, Read};

pub const PSQT_BUCKETS: usize = 8;

const PACKUS_EPI16_ORDER: [usize; 8] = [0, 2, 1, 3, 4, 6, 5, 7];

fn permute_weights(data: &mut [i16]) {
    const BLOCK_SIZE: usize = 16;
    const ORDER_SIZE: usize = 8;
    const CHUNK_SIZE: usize = BLOCK_SIZE * ORDER_SIZE;

    if data.len() < CHUNK_SIZE {
        return;
    }

    let mut buffer = vec![0i16; CHUNK_SIZE];

    let mut i = 0;
    while i + CHUNK_SIZE <= data.len() {
        for j in 0..ORDER_SIZE {
            let src = i + PACKUS_EPI16_ORDER[j] * BLOCK_SIZE;
            buffer[j * BLOCK_SIZE..(j + 1) * BLOCK_SIZE]
                .copy_from_slice(&data[src..src + BLOCK_SIZE]);
        }
        data[i..i + CHUNK_SIZE].copy_from_slice(&buffer);
        i += CHUNK_SIZE;
    }
}

pub struct FeatureTransformer {
    pub input_dims: usize,
    pub half_dims: usize,
    pub biases: AlignedBuffer<i16>,
    pub weights: AlignedBuffer<i16>,
    pub psqt_weights: AlignedBuffer<i32>,
}

impl FeatureTransformer {
    pub fn new(input_dims: usize, half_dims: usize) -> Self {
        Self {
            input_dims,
            half_dims,
            biases: AlignedBuffer::new(0),
            weights: AlignedBuffer::new(0),
            psqt_weights: AlignedBuffer::new(0),
        }
    }

    pub fn read_parameters<R: Read>(
        &mut self,
        reader: &mut R,
        skip_first_magic: bool,
    ) -> io::Result<()> {
        let mut biases_vec = read_leb128_i16_checked(reader, self.half_dims, !skip_first_magic)?;
        let mut weights_vec = read_leb128_i16(reader, self.half_dims * self.input_dims)?;
        let psqt_weights_vec = read_leb128_i32(reader, PSQT_BUCKETS * self.input_dims)?;

        permute_weights(&mut biases_vec);
        permute_weights(&mut weights_vec);

        for b in &mut biases_vec {
            *b = b.wrapping_mul(2);
        }
        for w in &mut weights_vec {
            *w = w.wrapping_mul(2);
        }

        self.biases = AlignedBuffer::from_vec(biases_vec);
        self.weights = AlignedBuffer::from_vec(weights_vec);
        self.psqt_weights = AlignedBuffer::from_vec(psqt_weights_vec);

        Ok(())
    }
}
