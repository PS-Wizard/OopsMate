use crate::arch::{
    DENSE_CHUNK_SIZE, FT_PERMUTE_BLOCK_I16S, FT_PERMUTE_GROUP_I16S, FT_PERMUTE_ORDER, FT_SCALE,
};

pub(crate) fn repack_feature_transformer(biases: &mut [i16], weights: &mut [i16]) {
    scale_i16s(biases);
    scale_i16s(weights);
    permute_ft_i16_blocks(biases);
    permute_ft_i16_blocks(weights);
}

pub(crate) fn repack_dense_weights(
    weights: &mut [i8],
    padded_input_dims: usize,
    output_dims: usize,
) {
    debug_assert_eq!(weights.len(), padded_input_dims * output_dims);
    debug_assert_eq!(padded_input_dims % DENSE_CHUNK_SIZE, 0);

    let mut packed = vec![0i8; weights.len()];
    for out in 0..output_dims {
        let row_offset = out * padded_input_dims;
        for input in 0..padded_input_dims {
            let packed_index =
                dense_packed_weight_index(input, out, padded_input_dims, output_dims);
            packed[packed_index] = weights[row_offset + input];
        }
    }

    weights.copy_from_slice(&packed);
}

#[inline(always)]
pub(crate) const fn dense_packed_weight_index(
    input: usize,
    output: usize,
    _padded_input_dims: usize,
    output_dims: usize,
) -> usize {
    let chunk = input / DENSE_CHUNK_SIZE;
    let lane = input % DENSE_CHUNK_SIZE;
    chunk * output_dims * DENSE_CHUNK_SIZE + output * DENSE_CHUNK_SIZE + lane
}

fn scale_i16s(values: &mut [i16]) {
    for value in values {
        *value = value.wrapping_mul(FT_SCALE);
    }
}

fn permute_ft_i16_blocks(values: &mut [i16]) {
    debug_assert_eq!(values.len() % FT_PERMUTE_GROUP_I16S, 0);

    let mut buffer = [0i16; FT_PERMUTE_GROUP_I16S];
    for chunk in values.chunks_exact_mut(FT_PERMUTE_GROUP_I16S) {
        for (dst_block, &src_block) in FT_PERMUTE_ORDER.iter().enumerate() {
            let dst_start = dst_block * FT_PERMUTE_BLOCK_I16S;
            let src_start = src_block * FT_PERMUTE_BLOCK_I16S;
            buffer[dst_start..dst_start + FT_PERMUTE_BLOCK_I16S]
                .copy_from_slice(&chunk[src_start..src_start + FT_PERMUTE_BLOCK_I16S]);
        }
        chunk.copy_from_slice(&buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::{dense_packed_weight_index, repack_dense_weights, repack_feature_transformer};

    #[test]
    fn dense_repack_groups_inputs_by_chunks_of_four() {
        let padded_input_dims = 8;
        let output_dims = 3;
        let mut weights: Vec<i8> = (0..(padded_input_dims * output_dims) as i8).collect();

        repack_dense_weights(&mut weights, padded_input_dims, output_dims);

        for out in 0..output_dims {
            for input in 0..padded_input_dims {
                let logical = (out * padded_input_dims + input) as i8;
                let packed = dense_packed_weight_index(input, out, padded_input_dims, output_dims);
                assert_eq!(weights[packed], logical);
            }
        }
    }

    #[test]
    fn feature_transformer_repack_scales_and_permutes_blocks() {
        let mut biases: Vec<i16> = (0..64).collect();
        let mut weights: Vec<i16> = (0..128).collect();

        repack_feature_transformer(&mut biases, &mut weights);

        assert_eq!(&biases[..8], &[0, 2, 4, 6, 8, 10, 12, 14]);
        assert_eq!(&biases[8..16], &[32, 34, 36, 38, 40, 42, 44, 46]);
        assert_eq!(&biases[16..24], &[16, 18, 20, 22, 24, 26, 28, 30]);
        assert_eq!(&weights[..8], &[0, 2, 4, 6, 8, 10, 12, 14]);
        assert_eq!(&weights[8..16], &[32, 34, 36, 38, 40, 42, 44, 46]);
        assert_eq!(&weights[16..24], &[16, 18, 20, 22, 24, 26, 28, 30]);
    }
}
