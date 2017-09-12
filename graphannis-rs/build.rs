extern crate cheddar;
extern crate crypto;

use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::string::String;
use std::panic;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use cheddar::Error;

fn generate_single_capi_code(module : &str) -> Result<String, Vec<Error>> {
    
    let code = (cheddar::Cheddar::new()
            .expect("could not read manifest")
            .module(module).expect(&format!("malformed module path for {}", module))
            .compile_code())?;
    Ok(code)
}

#[allow(unused_must_use)]
fn generage_capi_header(modules : Vec<&str>, out_file : &str) {

    let out_path = Path::new(out_file);

    // make sure the directories exist
    if let Some(parent_dir) = out_path.parent() {
        std::fs::create_dir_all(parent_dir);

        let mut id : String = String::from("annis_");
        id = id + out_path.file_stem().unwrap().to_str().unwrap_or(out_file);
        // compile header but do not write output
        let mut cheddar = cheddar::Cheddar::new()
            .expect("could not read manifest");

        for m in modules {
            let module_code = generate_single_capi_code(m);


            if module_code.is_ok() {
                let c = module_code.unwrap();
                cheddar.insert_code(&c);
            } else {
                let error_vector = module_code.unwrap_err();
                for e in error_vector {
                    println!("cargo:warning=Could not compile module '{}': {}", m, e);
                }
            }


        }

        let header = cheddar.compile(id.as_str());
        
        // do not overwrite file if equal to avoid an updated timestamp and unnecessary compiles
        let mut old_header = String::new();
        let old_header_file = std::fs::File::open(out_path);
        if old_header_file.is_ok() {
            old_header_file.unwrap().read_to_string(&mut old_header);
        } else {
            old_header = String::from("");
        }

        let new_header = header.unwrap_or(String::from(""));
        
        let mut old_hasher = Sha256::new();
        old_hasher.input_str(&old_header);

        let mut new_hasher = Sha256::new();
        new_hasher.input_str(&new_header);

        if old_hasher.result_str() != new_hasher.result_str() {
            let mut out_file = std::fs::File::create(out_path).expect("Can't write C header file");
            out_file.write_all(new_header.as_bytes());
        }
    
    }
}


fn main() {

    let result = panic::catch_unwind(|| {
        generage_capi_header(vec![
        "annis",
        "annis::util::c_api",
        "annis::stringstorage::c_api",
        "annis::annostorage::c_api"
        ],
        "include/graphannis-capi.h"); 
    });

    if result.is_err() {
        println!("cargo:warning=Auto-generated C header not re-generated because there are compile errors.");
    }

    
   
}