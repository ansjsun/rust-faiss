use crate::{Config, IndexIVFPQ};
use cpp::cpp;

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

}}

impl IndexIVFPQ {
    pub fn new(conf: &Config) -> Result<Self, String> {
        let (dimension, nprobe, ncentroids, bytes_per_code) = (
            conf.dimension,
            conf.nprobe,
            conf.ncentroids,
            conf.bytes_per_code,
        );

        let nhash = 2;
        //number of bit per subvector index
        let nbits_subq = ((ncentroids as f32).log2() / nhash as f32) as i32;

        let index = unsafe {
            cpp!([dimension as "int", nprobe as "int", nbits_subq as "int",nhash as "int", ncentroids as "size_t", bytes_per_code as "int"] ->  IndexIVFPQ as "faiss::IndexIVFPQ"{
                faiss::MultiIndexQuantizer *coarse_quantizer = new faiss::MultiIndexQuantizer(dimension, nhash, nbits_subq);
                faiss::IndexIVFPQ *index = new faiss::IndexIVFPQ(coarse_quantizer, dimension, ncentroids, bytes_per_code, 8);
                (*index).quantizer_trains_alone = true;
                (*index).verbose = true;
                (*index).nprobe = nprobe;
                return *index ;
            })
        };

        return Ok(index);
    }

    pub fn train(&self, trainvecs: &Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let train_size = trainvecs.len() as i32;
            cpp!([self as "faiss::IndexIVFPQ *", train_size as "int", trainvecs as "std::vector<float> *"]{
                size_t nt = train_size / self -> d ;
                self -> train(nt, trainvecs -> data());
            });
        }

        Ok(())
    }

    pub fn add(&self, num: usize, datavecs: &Vec<f32>) {
        unsafe {
            cpp!([self as "faiss::IndexIVFPQ *", num as "size_t", datavecs as "std::vector<float> *"] {
                self -> add(num, datavecs -> data()) ;
            })
        };
    }

    pub fn add_with_ids(
        &self,
        ids: &Vec<i64>,
        datavecs: &Vec<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let num = ids.len();
        unsafe {
            cpp!([self as "faiss::IndexIVFPQ *", num as "size_t", datavecs as "std::vector<float> *", ids as "std::vector<faiss::Index::idx_t> *"] {
                self -> add_with_ids(num, datavecs -> data(), ids -> data()) ;
            })
        };

        Ok(())
    }

    pub fn add_with_id(
        &self,
        id: i64,
        datavecs: &Vec<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.add_with_ids(&vec![id], datavecs)
    }

    // search index, return  id list , and score list . if result < size , it will truncate
    pub fn search(&self, size: i32, num_querys: i32, queries: &Vec<f32>) -> (Vec<i64>, Vec<f32>) {
        let len = (size * num_querys) as usize;
        let mut nns: Vec<i64> = Vec::with_capacity(len);
        let mut dis: Vec<f32> = Vec::with_capacity(len);
        unsafe {
            let (nns, dis) = (&mut nns, &mut dis);
            cpp!([self as "faiss::IndexIVFPQ *",num_querys as "int", size as "int" , queries  as "std::vector<float> *" ,
                nns as "std::vector<int64_t> *" , dis as "std::vector<float> *" ]{
                self -> search(num_querys, queries -> data(), size, dis -> data(), nns -> data());
            });
            nns.set_len(len);
            dis.set_len(len);
        };

        let mut temp = len - 1;
        loop {
            if nns[temp] == -1 {
                temp -= 1;
                continue;
            }
            break;
        }

        temp += 1;
        if len > temp {
            nns.truncate(temp);
            dis.truncate(temp);
        }

        return (nns, dis);
    }
}

#[test]
fn test_ivf_pd_add() {
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
    let index = IndexIVFPQ::new(&conf).unwrap();

    index.train(&vec).unwrap();

    println!("========= test add");
    let mut vec = Vec::with_capacity(dimension * index_size / 2);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }

    index.add(index_size / 2, &vec);

    println!("========= test add with id");
    let mut vec = Vec::with_capacity(dimension);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    index.add_with_id(99999999, &vec).unwrap();

    println!("========= test add with ids");
    let mid = index_size / 2 + 1;
    let mut vec = Vec::with_capacity(dimension * index_size / 2);
    let mut ids: Vec<i64> = Vec::with_capacity(vec.capacity());
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    for i in mid..index_size / 2 + 1 {
        ids.push(i as i64);
    }
    index.add_with_ids(&ids, &vec).unwrap();

    println!("========= test search");
    let mut vec = Vec::with_capacity(dimension);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    let result = index.search(2000, 1, &vec);
    println!("search result : {:?}", result);
}
