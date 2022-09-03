use crate::prelude::*;

use ndarray::Zip;

fn zip_sum<D, TIn, TOut, F>((a, b): (ArrayView<TIn, D>, ArrayView<TIn, D>), f: F) -> TOut
where
    D: Dimension,
    TIn: Num + Copy,
    TOut: Num + Copy,
    F: Fn(TIn, TIn) -> TOut,
{
    Zip::from(a).and(b).fold(TOut::zero(), |acc, &x, &y| acc + f(x, y))
}

pub type SimFn = fn((ArrayView1<u32>, ArrayView1<u32>)) -> f64;

pub fn jaccard((a, b): (ArrayView1<u32>, ArrayView1<u32>)) -> f64 {
    match zip_sum((a, b), std::cmp::max) {
        0 => 1f64,
        n => zip_sum((a, b), std::cmp::min) as f64 / n as f64,
    }
}

pub fn simmat(arr: ArrayView2<u32>, axis: Axis, f: SimFn) -> Array2<f64> {
    let values = iproduct!(arr.axis_iter(axis), arr.axis_iter(axis)).map(f);
    Array::from_shape_vec(arr.raw_dim(), values.collect()).unwrap()
}

pub fn adjmat<I: IntoIterator<Item = IdxPair>>(pairs: I) -> Array2<u8> {
    let pairs = pairs.into_iter().collect_vec();
    let max = pairs.iter().flat_map(|(a, b)| [a, b]).max().unwrap();
    let mut arr = Array2::zeros((max + 1, max + 1));

    for (src, tgt) in pairs {
        arr[[src, tgt]] = 1;
    }

    arr
}
