extern crate cheddar;

use std::io::Read;
use std::io::Write;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[allow(unused_must_use)]
fn main() {

    let out_path = "include/graphannis-api.h";

    // compile header but do not write output
    let header = cheddar::Cheddar::new()
        .expect("could not read manifest")
        .module("c_api").expect("malformed module path")
        .compile("graphannis-api");
    if header.is_ok() {
        // do not overwrite file if equal to avoid an updated timestamp and unnecessary compiles
        let mut old_header = String::new();
        let old_header_file = std::fs::File::open(out_path);
        if old_header_file.is_ok() {
            old_header_file.unwrap().read_to_string(&mut old_header);
        }

        let new_header = header.unwrap();
        
        let mut old_hasher = DefaultHasher::new();
        old_header.hash(&mut old_hasher);

        let mut new_hasher = DefaultHasher::new();
        new_header.hash(&mut new_hasher);

        if old_hasher.finish() != new_hasher.finish() {
            let mut out_file = std::fs::File::create(out_path).expect("Can't write C header file");
            out_file.write_all(new_header.as_bytes());
        } else {
            println!("cargo:warning=Auto-generated C header file did not change and is not re-generated");
        }
    

    }
}