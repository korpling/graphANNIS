#[macro_use]
extern crate bencher;

#[macro_use]
extern crate lazy_static;

extern crate graphannis;

use bencher::Bencher;
use std::path::PathBuf;

use graphannis::util;
use graphannis::api::corpusstorage::CorpusStorage;

lazy_static! {
    static ref CS : CorpusStorage =  {
        let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("../data")
        });
        CorpusStorage::new(&db_dir).unwrap()    
    };
}

fn get_query_dir() -> PathBuf {
    let query_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("../queries")
    });
    query_dir
}


struct GUM {
    pub def : util::SearchDef,
}

impl bencher::TDynBenchFn for GUM {

    #[allow(unused_must_use)]
    fn run(&self, bench: &mut Bencher) {

        CS.preload("GUM");

        bench.iter(|| {
                if let Ok(count) = CS.count("GUM", &self.def.json) {
                    assert_eq!(self.def.count, count);
                } else {
                    assert_eq!(self.def.count, 0);
                }
        });
    }
}

pub fn count_gum() -> std::vec::Vec<bencher::TestDescAndFn> {
    use bencher::{TestDescAndFn, TestFn, TestDesc};
    use std::borrow::Cow;

    
    let mut d = get_query_dir();
    d.push("SearchTestGUM");

    let queries = util::get_queries_from_folder(&d, true);

    let mut benches = std::vec::Vec::new();

    for def in queries {

        benches.push(TestDescAndFn {
            desc: TestDesc {
                name: Cow::from(def.name.clone()),
                ignore: false,
            },
            testfn: TestFn::DynBenchFn(Box::new(GUM{def})),
        });
    }

    benches
}

benchmark_main!(count_gum);