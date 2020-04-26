#![recursion_limit = "512"]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use cpp::cpp;
use cpp::cpp_class;
use std::any::Any;

mod index_ivf_pq;

cpp! {{
    #include <faiss/IndexIVFPQ.h>
    #include <faiss/IndexIVFFlat.h>
}}

cpp_class!(pub unsafe struct IndexIVFFlat as "faiss::IndexIVFFlat");
cpp_class!(pub unsafe struct IndexIVFPQ as "faiss::IndexIVFPQ");

pub trait Index {
    fn new(conf: &Config) -> Self;
}

pub struct Faiss<T> {
    pub config: Config,
    index: T,
}

impl<T: Any + Index> Faiss<T> {
    pub fn new(conf: Config) -> Self {
        let index = T::new(&conf);

        Faiss {
            config: conf,
            index: index,
        }
    }
    pub fn add(&self, num: usize, datavecs: &Vec<f32>) {
        let index = &self.index;
        unsafe {
            cpp!([index as "faiss::IndexIVFPQ *", num as "size_t", datavecs as "std::vector<float> *"] {
                index -> add(num, datavecs -> data()) ;
            })
        };
    }

    pub fn train(&self, trainvecs: &Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
        let index = &self.index;
        unsafe {
            let train_size = trainvecs.len() as i32;
            cpp!([index as "faiss::Index *", train_size as "int", trainvecs as "std::vector<float> *"]{
                size_t nt = train_size / index -> d ;
                index -> train(nt, trainvecs -> data());
            });
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
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
    // the type of metric
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
