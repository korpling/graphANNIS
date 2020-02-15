#[macro_use]
extern crate criterion;
#[macro_use]
extern crate lazy_static;
extern crate graphannis;
extern crate rand;
extern crate rustc_hash;

use criterion::Criterion;
use graphannis::corpusstorage::QueryLanguage;
use graphannis::corpusstorage::ResultOrder;
use graphannis::util::DiskMap;
use graphannis::CorpusStorage;
use std::collections::HashSet;
use std::path::PathBuf;

use fake::{Fake, Faker};

lazy_static! {

static ref CORPUS_STORAGE : Option<CorpusStorage> = {
    let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
        path
    } else {
        String::from("data")
    });

    // only execute the test if the directory exists
    let cs = if db_dir.exists() && db_dir.is_dir() {
        CorpusStorage::with_auto_cache_size(&db_dir, false).ok()
    } else {
        None
    };
    return cs;
    };
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
                &["GUM"],
                "pos=\"NN\"",
                QueryLanguage::AQL,
                usize::min_value(),
                None,
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

fn insert_disk_map(bench: &mut Criterion) {
    bench.bench_function("insert_disk_map", move |b| {
        let mut data = Vec::default();
        for _ in 0..100_000 {
            data.push((Faker.fake::<u32>(), Faker.fake::<String>()));
        }
        b.iter(|| {
            let mut m = DiskMap::default();
            for (key, value) in &data {
                m.insert(*key, value.clone()).unwrap();
            }
        });
    });
}

criterion_group!(name=corpusstorage; config= Criterion::default().sample_size(25); targets = find_all_nouns_gum);
criterion_group!(name=serialization; config= Criterion::default().sample_size(25); targets = deserialize_gum);

criterion_group!(name=diskmap; config= Criterion::default().sample_size(10); targets = insert_disk_map);

criterion_main!(corpusstorage, serialization, diskmap);
