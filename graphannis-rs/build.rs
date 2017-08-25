extern crate cheddar;

#[allow(unused_must_use)]
fn main() {
    cheddar::Cheddar::new()
        .expect("could not read manifest")
        .module("c_api").expect("malformed module path")
        .write("include/graphannis-api.h");
//        .run_build("include/graphannis-api.h");
}