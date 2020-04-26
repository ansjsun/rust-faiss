use crate::{Config, Index, IndexIVFPQ};
use cpp::cpp;
use cpp::cpp_class;

cpp! {{
    #include <iostream>

    #include <cmath>
    #include <cstdio>
    #include <cstdlib>

    #include <sys/time.h>

    #include <faiss/IndexPQ.h>
    #include <faiss/IndexIVFPQ.h>
    #include <faiss/IndexFlat.h>
    #include <faiss/index_io.h>
    #include <faiss/index_factory.h>
    #include <faiss/MetaIndexes.h>

}}

cpp_class!(pub unsafe struct IndexIDMap as "faiss::IndexIDMap");

impl Index for IndexIVFPQ {
    fn new(conf: &Config) -> Self {
        let (dimension, nprobe, ncentroids, bytes_per_code) = (
            conf.dimension,
            conf.nprobe,
            conf.ncentroids,
            conf.bytes_per_code,
        );

        let index = unsafe {
            cpp!([] -> IndexIDMap as "faiss::IndexIDMap"{
                auto index = index_factory(128,"PCA32,IVF100,PQ8", faiss::METRIC_L2);
                faiss::IndexIDMap *map = new faiss::IndexIDMap(index);
                return *map ;
            })
        };

        panic!()

        // let nhash = 2;
        // //number of bit per subvector index
        // let nbits_subq = ((ncentroids as f32).log2() / nhash as f32) as i32;

        // let index = unsafe {
        //     cpp!([dimension as "int", nprobe as "int", nbits_subq as "int",nhash as "int", ncentroids as "size_t", bytes_per_code as "int" ] ->  IndexIVFPQ as "faiss::IndexIVFPQ"{
        //         faiss::MultiIndexQuantizer *coarse_quantizer = new faiss::MultiIndexQuantizer(dimension, nhash, nbits_subq);
        //         faiss::IndexIVFPQ *index = new faiss::IndexIVFPQ(coarse_quantizer, dimension, ncentroids, bytes_per_code, 8);
        //         (*index).quantizer_trains_alone = true;
        //         (*index).verbose = true;
        //         (*index).nprobe = nprobe;
        //         return *index ;
        //     })
        // };
        // return index;
    }
}

#[test]
fn test_ivf_pd_add() {
    use crate::Faiss;
    use rand;

    let dimension: usize = 128;
    let index_size = 100000;
    let train_size = 10000;

    let conf = Config::new(dimension as i32);

    let mut vec = Vec::with_capacity(dimension * train_size);
    for _ in 0..vec.capacity() {
        let v = rand::random::<f32>();
        vec.push(v);
    }
    let index: Faiss<IndexIVFPQ> = Faiss::new(conf);

    index.train(&vec).unwrap();

    println!("========= test add");
    let mut vec = Vec::with_capacity(dimension * index_size / 2);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }

    index.add(index_size / 2, &vec);

    // println!("========= test add with id");
    // let mut vec = Vec::with_capacity(dimension);
    // for _i in 0..vec.capacity() {
    //     vec.push(rand::random::<f32>());
    // }
    // index.add_with_id(99999999, &vec).unwrap();

    // println!("========= test add with ids");
    // let mid = index_size / 2 + 1;
    // let mut vec = Vec::with_capacity(dimension * index_size / 2);
    // let mut ids: Vec<i64> = Vec::with_capacity(vec.capacity());
    // for _i in 0..vec.capacity() {
    //     vec.push(rand::random::<f32>());
    // }
    // for i in mid..index_size / 2 + 1 {
    //     ids.push(i as i64);
    // }
    // index.add_with_ids(&ids, &vec).unwrap();

    // println!("========= test search");
    // let mut vec = Vec::with_capacity(dimension);
    // for _i in 0..vec.capacity() {
    //     vec.push(rand::random::<f32>());
    // }
    // let result = index.search(2000, 1, &vec);
    // println!("search result : {:?}", result);
}
