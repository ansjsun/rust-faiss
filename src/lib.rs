#![recursion_limit = "512"]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use cpp::{cpp, cpp_class};
use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    RwLock,
};

cpp! {{
    #include <iostream>

    #include <cmath>
    #include <cstdio>
    #include <cstdlib>

    #include <sys/time.h>


    #include <faiss/IndexPQ.h>
    #include <faiss/IndexIVFFlat.h>
    #include <faiss/IndexFlat.h>
    #include <faiss/index_io.h>

}}

cpp_class!(unsafe struct IndexIVFFlat as "faiss::IndexIVFFlat");

pub struct Faiss {
    index: Option<IndexIVFFlat>,
    dimension: i32,
    index_size: usize,
}

impl Faiss {
    pub fn new(dimension: i32, index_size: usize) -> Faiss {
        Faiss {
            dimension: dimension,
            index_size: index_size,
            index: None,
        }
    }

    pub fn train(&mut self, trainvecs: Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
        let dimension = self.dimension;
        let index_size = self.index_size;
        let train_size = trainvecs.len() / dimension as usize;
        let mut aaaa: IndexIVFFlat = unsafe {
            cpp!([dimension as "int",index_size as "size_t",train_size as "size_t", trainvecs as "std::vector<float>"] ->  IndexIVFFlat as "faiss::IndexIVFFlat"{

                size_t nhash = 2;
                size_t nbits_subq = int (log2 (index_size+1) / 2);
                size_t ncentroids = 1 << (nhash * nbits_subq);
                faiss::MultiIndexQuantizer coarse_quantizer (dimension, nhash, nbits_subq);
                faiss::MetricType metric = faiss::METRIC_L2;
                faiss::IndexIVFFlat *index = new faiss::IndexIVFFlat(&coarse_quantizer, dimension, ncentroids, metric);
                (*index).quantizer_trains_alone = true;
                (*index).verbose = true;
                (*index).train(train_size, trainvecs.data());

                return *index ;
            })
        };

        unsafe {
            cpp!([aaaa as "faiss::IndexIVFFlat", train_size as "size_t", trainvecs as "std::vector<float>"] {
                // index.train(train_size, trainvecs.data());
            });
        }

        // self.index = Some(index);

        Ok(())
    }

    // pub fn add_all(
    //     self,
    //     ids: Vec<u64>,
    //     datavecs: Vec<f32>,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let num = ids.len();
    //     let mut index = self.index.unwrap();
    //     unsafe {
    //         cpp!([index as "faiss::IndexIVFFlat", num as "size_t", datavecs as "std::vector<float>", ids as "std::vector<faiss::Index::idx_t>"] {
    //             //index.add_with_ids(num, datavecs.data(), ids.data()) ;
    //         })
    //     };

    //     Ok(())
    // }

    // pub fn add(&self, id: u64, datavecs: Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
    //     self.add_all(vec![id], datavecs)
    // }
}

#[test]
fn test_new() {
    use rand;

    let dimension = 128;
    let index_size = 1000000;
    let train_size = 10000;

    let mut f = Faiss::new(dimension as i32, index_size);

    let mut vec = Vec::with_capacity(dimension * train_size);

    for i in 0..vec.capacity() {
        let v = rand::random::<f32>();
        vec.push(v);
    }

    f.train(vec);

    // let mut vec: Vec<f32> = Vec::with_capacity(dimension);
    // for i in (0..vec.capacity()) {
    //     let v = rand::random::<f32>();
    // }

    // let query = vec.clone();

    // f.add_all(vec![1], vec);

    // println!("{:?}", query);
}
