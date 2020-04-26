#![recursion_limit = "512"]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use cpp::cpp;
use cpp::cpp_class;

cpp! {{
    #include <faiss/index_factory.h>
    #include <faiss/MetaIndexes.h>
}}

cpp_class!(unsafe struct IndexIDMap as "faiss::IndexIDMap");

pub struct Faiss {
    pub config: Config,
    index: IndexIDMap,
}

impl Faiss {
    pub fn new(conf: Config) -> Self {
        let (dimension, description, metric) = (
            conf.dimension,
            conf.description.as_ptr(),
            conf.metric_type as i32,
        );
        let index = unsafe {
            cpp!([dimension as "int", description as "const char *", metric as "faiss::MetricType"] -> IndexIDMap as "faiss::IndexIDMap"{
                auto index = index_factory(dimension,description, metric);
                faiss::IndexIDMap *map = new faiss::IndexIDMap(index);
                return *map ;
            })
        };

        Faiss {
            config: conf,
            index: index,
        }
    }

    pub fn add_with_id(
        &self,
        id: i64,
        datavecs: &Vec<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.add_with_ids(&vec![id], datavecs)
    }

    pub fn add_with_ids(
        &self,
        ids: &Vec<i64>,
        datavecs: &Vec<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let num = ids.len();
        let index = &self.index;
        unsafe {
            cpp!([index as "faiss::IndexIDMap *", num as "size_t", datavecs as "std::vector<float> *", ids as "std::vector<faiss::Index::idx_t> *"] {
                index -> add_with_ids(num, datavecs -> data(), ids -> data()) ;
            })
        };

        Ok(())
    }

    pub fn train(&self, trainvecs: &Vec<f32>) -> Result<(), Box<dyn std::error::Error>> {
        let index = &self.index;
        unsafe {
            let train_size = trainvecs.len() as i32;
            cpp!([index as "faiss::IndexIDMap *", train_size as "int", trainvecs as "std::vector<float> *"]{
                size_t nt = train_size / index -> d ;
                index -> train(nt, trainvecs -> data());
            });
        }
        Ok(())
    }

    // search index, return  id list , and score list . if result < size , it will truncate
    pub fn search(&self, size: i32, num_querys: i32, queries: &Vec<f32>) -> (Vec<i64>, Vec<f32>) {
        let index = &self.index;
        let len = (size * num_querys) as usize;
        let mut nns: Vec<i64> = Vec::with_capacity(len);
        let mut dis: Vec<f32> = Vec::with_capacity(len);
        unsafe {
            let (nns, dis) = (&mut nns, &mut dis);
            cpp!([index as "faiss::IndexIDMap *",num_querys as "int", size as "int" , queries  as "std::vector<float> *" ,
                nns as "std::vector<int64_t> *" , dis as "std::vector<float> *" ]{
                index -> search(num_querys, queries -> data(), size, dis -> data(), nns -> data());
            });
            nns.set_len(len);
            dis.set_len(len);
        };

        let mut size = 0;
        let mut temp = len - 1;
        loop {
            if nns[temp] == -1 {
                temp -= 1;
                if temp == 0 {
                    size = -1;
                    break;
                }
                continue;
            }
            size = temp as i32;
            break;
        }

        temp = (size + 1) as usize;
        if len > temp {
            nns.truncate(temp);
            dis.truncate(temp);
        }

        return (nns, dis);
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
    // A constructor
    pub description: String,
    // the type of metric
    pub metric_type: MetricType,
}

impl Config {
    pub fn new(dimension: i32) -> Config {
        return Config {
            dimension: dimension,
            description: String::from("PCA32,IVF100,PQ8"),
            metric_type: MetricType::L2,
        };
    }
}

#[test]
fn test_default() {
    use rand;
    let dimension: usize = 128;
    let index_size = 100000;
    let train_size = 10000;

    let mut conf = Config::new(dimension as i32);
    conf.description = String::from("PCA32,IVF1,PQ8");
    let faiss = Faiss::new(conf);

    let mut vec = Vec::with_capacity(dimension * train_size);
    for _ in 0..vec.capacity() {
        let v = rand::random::<f32>();
        vec.push(v);
    }
    faiss.train(&vec).unwrap();

    println!("========= test add with id");

    for i in 0..2 {
        let mut vec = Vec::with_capacity(dimension);
        for _i in 0..vec.capacity() {
            vec.push(rand::random::<f32>());
        }
        faiss.add_with_id(i * 9, &vec).unwrap();
    }

    println!("========= test add with ids");
    let mut vec = Vec::with_capacity(dimension * index_size / 2);
    let mut ids: Vec<i64> = Vec::with_capacity(vec.capacity());
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    for i in 0..index_size {
        ids.push(i as i64);
    }
    faiss.add_with_ids(&ids, &vec).unwrap();

    println!("========= test search");
    let mut vec = Vec::with_capacity(dimension);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    let result = faiss.search(2000, 1, &vec);
    println!("search result : {:?}", result);
}
