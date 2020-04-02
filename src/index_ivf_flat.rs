use crate::IndexIVFFlat;
use cpp::cpp;

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

impl IndexIVFFlat {
    pub fn new(dimension: i32) -> Self {
        let doc_size = 1_000_000;
        unsafe {
            cpp!([dimension as "int", doc_size as "int"] ->  IndexIVFFlat as "faiss::IndexIVFFlat"{
                size_t nhash = 2;
                size_t nbits_subq = int (log2 (doc_size+1) / 2);
                size_t ncentroids = 1 << (nhash * nbits_subq);
                faiss::MultiIndexQuantizer *coarse_quantizer = new faiss::MultiIndexQuantizer(dimension, nhash, nbits_subq);
                faiss::MetricType metric = faiss::METRIC_L2;
                faiss::IndexIVFFlat *index = new faiss::IndexIVFFlat(coarse_quantizer, dimension, ncentroids, metric);
                (*index).quantizer_trains_alone = true;
                (*index).verbose = true;
                (*index).nprobe = 2048;
                return *index ;
            })
        }
    }

    pub fn train(&self, trainvecs: Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let train_size = trainvecs.len() as i32;
            cpp!([self as "faiss::IndexIVFFlat *", train_size as "int", trainvecs as "std::vector<float>"]{
                size_t nt = train_size / self -> d ;
                printf("[%d] [%d] [%d]",train_size,self -> d, nt);
                self -> train(nt, trainvecs.data());
            });
        }

        Ok(())
    }

    pub fn add(&self, num: usize, datavecs: Vec<f32>) {
        unsafe {
            cpp!([self as "faiss::IndexIVFFlat *", num as "size_t", datavecs as "std::vector<float>"] {
                self -> add(num, datavecs.data()) ;
            })
        };
    }

    pub fn add_with_ids(
        &self,
        ids: Vec<i64>,
        datavecs: Vec<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let num = ids.len();
        unsafe {
            cpp!([self as "faiss::IndexIVFFlat *", num as "size_t", datavecs as "std::vector<float>", ids as "std::vector<faiss::Index::idx_t>"] {
                self -> add_with_ids(num, datavecs.data(), ids.data()) ;
            })
        };

        Ok(())
    }

    pub fn add_with_id(
        &self,
        id: i64,
        datavecs: Vec<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.add_with_ids(vec![id], datavecs)
    }

    // search index, return  id list , and score list . if result < size , it will truncate
    pub fn search(&self, size: i32, num_querys: i32, queries: &Vec<f32>) -> (Vec<i64>, Vec<f32>) {
        let len = (size * num_querys) as usize;
        let mut nns: Vec<i64> = Vec::with_capacity(len);
        let mut dis: Vec<f32> = Vec::with_capacity(len);
        unsafe {
            let (nns, dis) = (&mut nns, &mut dis);
            cpp!([self as "faiss::IndexIVFFlat *",num_querys as "int", size as "int" , queries  as "std::vector<float> *" ,
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
fn test_ivf_flat_add() {
    use rand;

    let dimension = 128;
    let index_size = 100000;
    let train_size = 10000;

    let mut vec = Vec::with_capacity(dimension * train_size);
    for _ in 0..vec.capacity() {
        let v = rand::random::<f32>();
        vec.push(v);
    }
    let index = IndexIVFFlat::new(128);

    index.train(vec).unwrap();

    println!("========= test add");
    let mut vec = Vec::with_capacity(dimension * index_size / 2);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }

    index.add(index_size / 2, vec);

    println!("========= test add with id");
    let mut vec = Vec::with_capacity(dimension);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    index.add_with_id(99999999, vec).unwrap();

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
    index.add_with_ids(ids, vec).unwrap();

    println!("========= test search");
    let mut vec = Vec::with_capacity(dimension);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    let result = index.search(2000, 1, &vec);
    println!("search result : {:?}", result);
}
