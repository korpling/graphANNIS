#[macro_use]
extern crate bencher;
extern crate fxhash;
extern crate graphannis;
extern crate rand;

use bencher::Bencher;
use graphannis::annostorage::AnnoStorage;
use graphannis::api::corpusstorage::CorpusStorage;
use graphannis::api::corpusstorage::ResultOrder;
use graphannis::{AnnoKey, Annotation, StringID};
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;

use rand::distributions::{IndependentSample, Range};
use rand::seq;

fn retrieve_annos_for_node(bench: &mut Bencher) {
    let mut annos: AnnoStorage<usize> = AnnoStorage::new();

    let mut rng = rand::thread_rng();

    let ns_range: Range<StringID> = Range::new(0, 3);
    let name_range: Range<StringID> = Range::new(0, 20);
    let val_range: Range<StringID> = Range::new(0, 100);

    // insert 10 000 random annotations
    for i in 0..10_000 {
        let a = Annotation {
            key: AnnoKey {
                ns: ns_range.ind_sample(&mut rng),
                name: name_range.ind_sample(&mut rng),
            },
            val: val_range.ind_sample(&mut rng),
        };
        annos.insert(i, a);
    }

    // sample 1000 items to get the annotation value from
    let samples: Vec<usize> = seq::sample_indices(&mut rng, 10_000, 1_000);

    bench.iter(move || {
        let mut sum = 0;
        for i in samples.iter() {
            let a = annos.get_all(i);
            sum += a.len();
        }
        assert!(sum > 0);
    })
}

thread_local!{
   pub static CORPUS_STORAGE : RefCell<Option<CorpusStorage>> = {
        let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("../data")
        });

        // only execute the test if the directory exists
        let cs = if db_dir.exists() && db_dir.is_dir() {
            CorpusStorage::new_auto_cache_size(&db_dir, false).ok()
        } else {
            None
        };
        return RefCell::new(cs);
       };
}

fn find_all_nouns_gum(bench: &mut Bencher) {
    CORPUS_STORAGE.with(|cs| {
        if let Some(ref cs) = *cs.borrow() {
            if let Ok(corpora) = cs.list() {
                let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
                // ignore of corpus does not exist
                if corpora.contains("GUM") {
                    bench.iter(move || {
                        let f = cs.find("GUM", "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"nodeAnnotations\":[{\"name\":\"pos\",\"value\":\"NN\",\"textMatching\":\"EXACT_EQUAL\",\"qualifiedName\":\"pos\"}],\"root\":false,\"token\":false,\"variable\":\"1\"}},\"joins\":[]}]}", usize::min_value(), usize::max_value(), ResultOrder::Normal);
                        assert!(f.is_ok());
                    });
                }
            }
        }
    });
}

benchmark_group!(annostorage, retrieve_annos_for_node);

benchmark_group!(corpusstorage, find_all_nouns_gum);

benchmark_main!(annostorage, corpusstorage);
