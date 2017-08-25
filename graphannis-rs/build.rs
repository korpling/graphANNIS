extern crate cheddar;
extern crate crypto;

use std::io::Read;
use std::io::Write;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

fn generage_capi_header(module : &str, out_path : &str) {

    // compile header but do not write output
    let header = cheddar::Cheddar::new()
        .expect("could not read manifest")
        .module(module).expect("malformed module path")
        .compile("graphannis-api");
    if header.is_ok() {
        // do not overwrite file if equal to avoid an updated timestamp and unnecessary compiles
        let mut old_header = String::new();
        let old_header_file = std::fs::File::open(out_path);
        if old_header_file.is_ok() {
            old_header_file.unwrap().read_to_string(&mut old_header);
        } else {
            old_header = String::from("");
        }

        let new_header = header.unwrap();
        
        let mut old_hasher = Sha256::new();
        old_hasher.input_str(&old_header);

        let mut new_hasher = Sha256::new();
        new_hasher.input_str(&new_header);

        if old_hasher.result_str() != new_hasher.result_str() {
            let mut out_file = std::fs::File::create(out_path).expect("Can't write C header file");
            out_file.write_all(new_header.as_bytes());
        } else {
            println!("cargo:warning=Auto-generated C header file did not change and is *not* re-generated");
        }
    }
}

#[allow(unused_must_use)]
fn main() {

    generage_capi_header("c_api","include/graphannis-api.h");
    
}