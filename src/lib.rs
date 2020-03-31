#![recursion_limit = "512"]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use cpp::cpp_class;

cpp_class!(pub unsafe struct IndexIVFFlat as "faiss::IndexIVFFlat");

pub mod index_ivf_flat;
