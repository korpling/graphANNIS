#[macro_use]
extern crate bencher;

extern crate graphannis;

use bencher::Bencher;
use std::path::PathBuf;
use std::sync::Arc;

use graphannis::util;
use graphannis::api::corpusstorage::CorpusStorage;

fn get_query_dir() -> PathBuf {
    let query_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("../queries")
    });
    query_dir
}

struct GUM {
    pub cs : Arc<CorpusStorage>,
    pub def : util::SearchDef,
}

impl bencher::TDynBenchFn for GUM {

    #[allow(unused_must_use)]
    fn run(&self, bench: &mut Bencher) {
        
        // plan query to make sure all needed components are in main memory
        self.cs.plan("GUM", &self.def.json);

        bench.iter(|| {
                if let Ok(count) = self.cs.count("GUM", &self.def.json) {
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

    let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
        path
    } else {
        String::from("../data")
    });
    let mut d = get_query_dir();
    d.push("SearchTestGUM");

    let queries = util::get_queries_from_folder(&d, true);

    let mut benches = std::vec::Vec::new();

    let cs = Arc::new(CorpusStorage::new(&db_dir).unwrap());
    cs.preload("GUM").unwrap();

    for def in queries {

        benches.push(TestDescAndFn {
            desc: TestDesc {
                name: Cow::from(def.name.clone()),
                ignore: false,
            },
            testfn: TestFn::DynBenchFn(Box::new(GUM{cs: cs.clone(), def})),
        });
    }

    benches
}

benchmark_main!(count_gum);