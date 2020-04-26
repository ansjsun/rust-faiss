// use crate::{Config, Index};
// use cpp::cpp;

// cpp! {{
//     #include <iostream>

//     #include <cmath>
//     #include <cstdio>
//     #include <cstdlib>

//     #include <sys/time.h>

//     #include <faiss/IndexPQ.h>
//     #include <faiss/Index.h>
//     #include <faiss/IndexFlat.h>
//     #include <faiss/index_io.h>

// }}

// impl Index {
//     pub fn new(conf: &Config) -> Result<Self, String> {
//         let (dimension, nprobe, ncentroids) = (conf.dimension, conf.nprobe, conf.ncentroids);

//         let nhash = 2;
//         //number of bit per subvector index
//         let nbits_subq = ((ncentroids as f32).log2() / nhash as f32) as i32;

//         let index = unsafe {
//             cpp!([dimension as "int", nprobe as "int", nbits_subq as "int",nhash as "int", ncentroids as "size_t"] ->  Index as "faiss::Index"{
//                 faiss::MetricType metric = faiss::METRIC_L2;
//                 faiss::Index *index = new faiss::Index(11, metric);
//                 return *index ;
//             })
//         };

//         return Ok(index);
//     }

//     pub fn add(&self, num: usize, datavecs: &Vec<f32>) {
//         unsafe {
//             cpp!([self as "faiss::Index *", num as "size_t", datavecs as "std::vector<float> *"] {
//                 self -> add(num, datavecs -> data()) ;
//             })
//         };
//     }

//     // search index, return  id list , and score list . if result < size , it will truncate
//     pub fn search(&self, size: i32, num_querys: i32, queries: &Vec<f32>) -> (Vec<i64>, Vec<f32>) {
//         let len = (size * num_querys) as usize;
//         let mut nns: Vec<i64> = Vec::with_capacity(len);
//         let mut dis: Vec<f32> = Vec::with_capacity(len);
//         unsafe {
//             let (nns, dis) = (&mut nns, &mut dis);
//             cpp!([self as "faiss::Index *",num_querys as "int", size as "int" , queries  as "std::vector<float> *" ,
//                 nns as "std::vector<int64_t> *" , dis as "std::vector<float> *" ]{
//                 self -> search(num_querys, queries -> data(), size, dis -> data(), nns -> data());
//             });
//             nns.set_len(len);
//             dis.set_len(len);
//         };

//         let mut temp = len - 1;
//         loop {
//             if nns[temp] == -1 {
//                 temp -= 1;
//                 continue;
//             }
//             break;
//         }

//         temp += 1;
//         if len > temp {
//             nns.truncate(temp);
//             dis.truncate(temp);
//         }

//         return (nns, dis);
//     }
// }
