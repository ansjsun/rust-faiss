# Faiss for rust

> This project provides Rust bindings to Faiss, the state-of-the-art vector search and clustering library.

````
[dependencies]
faiss4rs = "1.6.306"
````

* 1.6.3 is faiss version , and last xx is this project`s version 

you need compile faiss in you compile box and set it in `/usr/local/lib` or `/usr/lib` or env `LD_LIBRARY_PATH`


this lib can use in 'macos' or `linux` but not support `windows`


### example 

````
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
````



