#![recursion_limit = "512"]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use cpp::cpp_class;

cpp_class!(pub unsafe struct IndexIVFFlat as "faiss::IndexIVFFlat");
// cpp_class!(pub unsafe struct IndexIVFPQ as "faiss::IndexIVFPQ");

pub mod index_ivf_flat;
// pub mod index_ivf_pq;

pub struct Config {
    //dimension: dimension of the input vectors
    pub dimension: i32,
    //nporbe: number of probes at query time
    pub nprobe: i32,
    //nhash: number of subquantizers default is 8
    pub nhash: i32,
    //doc_size: Estimated number of documents，This is used to calculate nbits_subq , default 10_000_000
    pub doc_size: i32,
    //bytes_per_code: is determined by the memory constraint, the dataset will use nb * (bytes_per_code + 8) bytes. default is 16
    pub bytes_per_code: i32,
}

impl Config {
    pub fn new(dimension: i32) -> Config {
        return Config {
            dimension: dimension,
            nprobe: 1024,
            nhash: 8,
            doc_size: 10_000_000,
            bytes_per_code: 16,
        };
    }
}
