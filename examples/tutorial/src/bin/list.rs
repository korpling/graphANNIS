use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let corpora = cs.list().unwrap();
    let corpus_names: Vec<String> = corpora
        .into_iter()
        .map(|corpus_info| corpus_info.name)
        .collect();
    println!("{:?}", corpus_names);
}
