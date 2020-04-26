#![recursion_limit = "512"]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use cpp::cpp;
use cpp::cpp_class;

cpp! {{
    #include <faiss/IndexIVFPQ.h>
    #include <faiss/IndexIVFFlat.h>
}}

cpp_class!(pub unsafe struct IndexIVFFlat as "faiss::IndexIVFFlat");
cpp_class!(pub unsafe struct IndexIVFPQ as "faiss::IndexIVFPQ");

pub struct Faiss {
    pub config: Config,
    index: Index,
}

impl Faiss {
    pub fn add(&self, num: usize, datavecs: &Vec<f32>) {
        let index = self.index.unwrap();
        unsafe {
            cpp!([index as "faiss::Index *", num as "size_t", datavecs as "std::vector<float> *"] {
                index -> add(num, datavecs -> data()) ;
            })
        };
    }
}

enum Index {
    IndexIVFFlat(IndexIVFFlat),
    IndexIVFPQ(IndexIVFPQ),
}

pub enum MetricType {
    L2 = 1,
    InnerProduct = 2,
}

pub struct Config {
    //dimension: dimension of the input vectors
    pub dimension: i32,
    //nporbe: number of probes at query time
    pub nprobe: i32,
    //ncentroids :InvertedLists ncentroids size
    pub ncentroids: i64,
    //bytes_per_code: is determined by the memory constraint, the dataset will use nb * (bytes_per_code + 8) bytes. default is 16
    pub bytes_per_code: i32,

    pub metric_type: MetricType,
}

impl Config {
    pub fn new(dimension: i32) -> Config {
        return Config {
            dimension: dimension,
            nprobe: 1024,
            ncentroids: 65536,
            bytes_per_code: 16,
            metric_type: MetricType::L2,
        };
    }
}
