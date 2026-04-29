mod accum_i16;
mod psqt_i32;
mod simd;

#[cfg(test)]
mod tests;

pub(crate) use accum_i16::{
    accum_add, accum_add_into_both, accum_add_sub, accum_add1_sub1_into, accum_add1_sub1_into_both,
    accum_add1_sub2_into, accum_add1_sub2_into_both, accum_add2_sub1_into,
    accum_add2_sub1_into_both, accum_add2_sub2_into, accum_sub, accum_sub_into_both,
};
pub(crate) use psqt_i32::{
    psqt_add, psqt_add_into_both, psqt_add_sub, psqt_add1_sub1_into, psqt_add1_sub1_into_both,
    psqt_add1_sub2_into, psqt_add1_sub2_into_both, psqt_add2_sub1_into, psqt_add2_sub1_into_both,
    psqt_add2_sub2_into, psqt_sub, psqt_sub_into_both,
};
