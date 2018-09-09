#[macro_use]
extern crate criterion;
#[macro_use]
extern crate lazy_static;
extern crate fxhash;
extern crate graphannis;
extern crate rand;

use criterion::Criterion;
use graphannis::annostorage::AnnoStorage;
use graphannis::api::corpusstorage::CorpusStorage;
use graphannis::api::corpusstorage::ResultOrder;
use graphannis::{AnnoKey, Annotation, StringID};
use std::collections::HashSet;
use std::path::PathBuf;

use rand::distributions::Distribution;
use rand::distributions::Range;

use rand::seq;

lazy_static! {

static ref CORPUS_STORAGE : Option<CorpusStorage> = {
    let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
        path
    } else {
        String::from("data")
    });

    // only execute the test if the directory exists
    let cs = if db_dir.exists() && db_dir.is_dir() {
        CorpusStorage::new_auto_cache_size(&db_dir, false).ok()
    } else {
        None
    };
    return cs;
    };
}

fn retrieve_annos_for_node(bench: &mut Criterion) {
    let mut annos: AnnoStorage<usize> = AnnoStorage::new();

    let mut rng = rand::thread_rng();

    let ns_range: Range<StringID> = Range::new(0, 3);
    let name_range: Range<StringID> = Range::new(0, 20);
    let val_range: Range<StringID> = Range::new(0, 100);

    // insert 10 000 random annotations
    for i in 0..10_000 {
        let a = Annotation {
            key: AnnoKey {
                ns: ns_range.sample(&mut rng),
                name: name_range.sample(&mut rng),
            },
            val: val_range.sample(&mut rng),
        };
        annos.insert(i, a);
    }

    // sample 1000 items to get the annotation value from
    let samples: Vec<usize> = seq::sample_indices(&mut rng, 10_000, 1_000);

    bench.bench_function("retrieve_annos_for_node", move |b| {
        b.iter(|| {
            let mut sum = 0;
            for i in samples.iter() {
                let a = annos.get_all(i);
                sum += a.len();
            }
            assert!(sum > 0);
        })
    });
}

fn find_all_nouns_gum(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();
    
    let corpora = cs.list();
    if let Ok(corpora) = corpora {
        let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
        // ignore of corpus does not exist
        if corpora.contains("GUM") {
            cs.preload("GUM").unwrap();
        } else {
            return;
        }
    }



    bench.bench_function("find_all_nouns_gum", move |b| {
        b.iter(|| {
            let f = cs.find(
                "GUM",
                "pos=\"NN\"",
                usize::min_value(),
                usize::max_value(),
                ResultOrder::Normal,
            );
            assert!(f.is_ok());
        })
    });

}

fn deserialize_gum(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();
    
    bench.bench_function("deserialize_gum", move |b| {
        b.iter(|| {
            cs.unload("GUM");
            cs.preload("GUM").unwrap();
        });
    });
}

criterion_group!(annostorage, retrieve_annos_for_node);
criterion_group!(name=corpusstorage; config= Criterion::default().sample_size(25); targets = find_all_nouns_gum);
criterion_group!(name=serialization; config= Criterion::default().sample_size(25); targets = deserialize_gum);

criterion_main!(annostorage, corpusstorage, serialization);
