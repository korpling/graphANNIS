extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .process_file("src/annis/db/aql/parser.lalrpop")
        .unwrap();
}
