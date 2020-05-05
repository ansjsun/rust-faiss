#![recursion_limit = "512"]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]
#![allow(unused)]

#[macro_use]
extern crate cpp;

cpp! {{
    #include <stdio.h>
    #include <faiss/index_factory.h>
    #include <faiss/MetaIndexes.h>
    #include <faiss/index_io.h>
}}

cpp_class!(pub unsafe struct IndexIDMap as "faiss::IndexIDMap");

pub struct Index {
    pub config: Config,
    index: IndexIDMap,
}

impl Index {
    pub fn open_or_create(conf: Config) -> Self {
        if std::path::Path::new(conf.path.as_str()).is_file() {
            return Self {
                index: Self::read_index(conf.path.as_str()),
                config: conf,
            };
        } else {
            return Self::new(conf);
        }
    }

    pub fn new(conf: Config) -> Self {
        let (dimension, description, metric) = (
            conf.dimension,
            conf.description.as_str(),
            conf.metric_type as i32,
        );

        let description = std::ffi::CString::new(description).unwrap();
        let d_ptr = description.as_ptr();

        let index = unsafe {
            cpp!([dimension as "int", d_ptr as "const char *", metric as "faiss::MetricType"] -> IndexIDMap as "faiss::IndexIDMap"{
                auto index = index_factory(dimension,d_ptr, metric);
                faiss::IndexIDMap *map = new faiss::IndexIDMap(index);
                return *map ;
            })
        };

        Index {
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

    pub fn is_trained(&self) -> bool {
        let index = &self.index;
        unsafe {
            cpp!([index as "faiss::IndexIDMap *"] -> bool as "bool" {
                return index -> is_trained;
            })
        }
    }

    pub fn dimension(&self) -> i32 {
        let index = &self.index;
        unsafe {
            cpp!([index as "faiss::IndexIDMap *"] -> i32 as "int" {
                return index -> d;
            })
        }
    }

    pub fn count(&self) -> i64 {
        let index = &self.index;
        unsafe {
            cpp!([index as "faiss::IndexIDMap *"] -> i64 as "long long" {
                return index -> ntotal;
            })
        }
    }

    pub fn max_id(&self) -> i64 {
        let index = &self.index;

        if self.count() == 0 {
            return 0;
        }

        unsafe {
            cpp!([index as "faiss::IndexIDMap *"] -> i64 as "long long" {
                long long v = index -> id_map.size();
                return index -> id_map[v-1] ;
            })
        }
    }

    // search index, return  id list , and score list . if result < size , it will truncate
    pub fn search(&self, size: i32, num_querys: i32, queries: &Vec<f32>) -> (Vec<i64>, Vec<f32>) {
        let index = &self.index;

        if self.count() == 0 {
            return (Vec::new(), Vec::new());
        }

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

    pub fn write_index(&self) {
        let index = &self.index;
        let path = std::ffi::CString::new(self.config.path.as_str()).unwrap();
        let path_ptr = path.as_ptr();
        unsafe {
            cpp!([index as "faiss::IndexIDMap *" , path_ptr as "const char *"] {
                faiss::write_index(index, path_ptr);
            })
        }
    }

    pub fn read_index(path: &str) -> IndexIDMap {
        let path = std::ffi::CString::new(path).unwrap();
        let path_ptr = path.as_ptr();
        unsafe {
            cpp!([path_ptr as "const char *"] -> IndexIDMap as "faiss::IndexIDMap" {
                auto index = faiss::read_index(path_ptr, 0);
                auto index_ref = dynamic_cast<faiss::IndexIDMap*>(index);
                return *index_ref ;
            })
        }
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
    // index path for persistence
    pub path: String,
}

impl Config {
    pub fn new(dimension: i32) -> Config {
        return Config {
            dimension: dimension,
            description: String::from("PCA32,IVF100,PQ8"),
            metric_type: MetricType::L2,
            path: String::from("temp.index"),
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
    conf.path = String::from("temp/large.index");
    conf.description = String::from("PCA32,IVF1,PQ8");
    let index = Index::new(conf);

    assert!(index.dimension() == dimension as i32);

    println!("========= dimension {}", index.dimension());
    println!("========= test train");
    let mut vec = Vec::with_capacity(dimension * train_size);
    for _ in 0..vec.capacity() {
        let v = rand::random::<f32>();
        vec.push(v);
    }

    assert!(!index.is_trained());
    println!("is_trained:{}", index.is_trained());
    index.train(&vec).unwrap();

    println!("========= test add with id");

    for i in 0..200 {
        let mut vec = Vec::with_capacity(dimension);
        for _i in 0..vec.capacity() {
            vec.push(rand::random::<f32>());
        }
        index.add_with_id(i * 9, &vec).unwrap();
    }

    println!("========= test add with ids");
    let mut vec = Vec::with_capacity(dimension * index_size);
    let mut ids: Vec<i64> = Vec::with_capacity(vec.capacity());
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    for i in 0..index_size {
        ids.push(i as i64);
    }
    // index.add_with_ids(&ids, &vec).unwrap();

    println!("========= test search");
    let mut vec = Vec::with_capacity(dimension);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    let result = index.search(2000, 1, &vec);
    println!("search result : {:?}", result);

    assert!(index.is_trained());
    println!("is_trained:{}", index.is_trained());

    assert_eq!(200, index.count());
    println!("all count:{}", index.count());

    println!("max_id:{}", index.max_id());

    std::fs::create_dir_all("temp");

    //to persistence index
    index.write_index();

    drop(index);

    let mut conf = Config::new(dimension as i32);
    conf.path = String::from("temp/large.index");

    let index = Index::open_or_create(conf);

    println!("========= test search");
    let mut vec = Vec::with_capacity(dimension);
    for _i in 0..vec.capacity() {
        vec.push(rand::random::<f32>());
    }
    let result = index.search(2000, 1, &vec);
    println!("search result : {:?}", result);

    assert!(index.is_trained());
    println!("is_trained:{}", index.is_trained());

    assert_eq!(200, index.count());
    println!("all count:{}", index.count());

    println!("max_id:{}", index.max_id());
}

#[test]
fn test_empty_need_train() {
    use rand;
    let dimension: usize = 128;
    let index_size = 100000;
    let train_size = 10000;

    let mut conf = Config::new(dimension as i32);
    conf.path = String::from("temp/empty.index");

    let index = Index::open_or_create(conf);

    assert!(!index.is_trained());
    println!("is_trained:{}", index.is_trained());

    assert_eq!(0, index.count());
    println!("all count:{}", index.count());

    assert_eq!(0, index.max_id());
    println!("max_id:{}", index.max_id());
}

#[test]
fn test_empty() {
    for desc in ["HNSW2", "Flat"].iter() {
        use rand;
        let dimension: usize = 128;
        let index_size = 100000;
        let train_size = 10000;
        let mut conf = Config::new(dimension as i32);
        conf.path = String::from("temp/empty.index");
        conf.description = desc.to_string();
        let index = Index::open_or_create(conf);

        println!("========= test search");
        let mut vec = Vec::with_capacity(dimension);
        for _i in 0..vec.capacity() {
            vec.push(rand::random::<f32>());
        }

        let result = index.search(2000, 1, &vec);
        println!("search result : {:?}", result);
        assert!(index.is_trained());
        println!("is_trained:{}", index.is_trained());
        assert_eq!(0, index.count());
        println!("all count:{}", index.count());
        assert_eq!(0, index.max_id());
        println!("max_id:{}", index.max_id());
    }
}
