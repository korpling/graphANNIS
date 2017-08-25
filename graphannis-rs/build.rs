extern crate cheddar;
extern crate crypto;

use std::path::Path;
use std::io::Read;
use std::io::Write;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

#[allow(unused_must_use)]
fn generage_capi_header(module : &str, out_file : &str) {

    let out_path = Path::new(out_file);

    // make sure the directories exist
    if let Some(parent_dir) = out_path.parent() {
        std::fs::create_dir_all(parent_dir);

        let mut id : String = String::from("annis_");
        id = id + out_path.file_stem().unwrap().to_str().unwrap_or(out_file);
        // compile header but do not write output
        let header = cheddar::Cheddar::new()
            .expect("could not read manifest")
            .module(module).expect("malformed module path")
            .compile(id.as_str());
        
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


fn main() {

    generage_capi_header("annis::stringstorage::c_api","include/graphannis/stringstorage.h");
    
}