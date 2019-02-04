extern crate clap;
extern crate criterion;

extern crate graphannis;

use clap::*;
use criterion::Bencher;
use criterion::Criterion;
use std::path::{Path, PathBuf};
use std::time::Duration;

use std::sync::Arc;

use graphannis::corpusstorage::QueryLanguage;
use graphannis::util;
use graphannis::CorpusStorage;

pub struct CountBench {
    pub def: util::SearchDef,
    pub cs: Arc<CorpusStorage>,
}

impl std::fmt::Debug for CountBench {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.def.corpus[0], self.def.name)
    }
}

pub fn create_query_input(
    data_dir: &Path,
    queries_file: &Path,
    use_parallel_joins: bool,
) -> std::vec::Vec<CountBench> {
    let mut benches = std::vec::Vec::new();

    let cs = Arc::new(CorpusStorage::with_auto_cache_size(data_dir, use_parallel_joins).unwrap());

    let queries = util::get_queries_from_csv(queries_file, true);
    for def in queries {
        let mut bench_name = String::from(def.corpus[0].clone());
        bench_name.push_str("/");
        bench_name.push_str(&def.name);

        benches.push(CountBench {
            def,
            cs: cs.clone(),
        });
    }

    return benches;
}

fn main() {
    let matches = App::new("graphANNIS search benchmark")
        .arg(
            Arg::with_name("output-dir")
                .long("output-dir")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("data")
                .long("data")
                .short("d")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("queries")
                .long("queries")
                .short("q")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("parallel")
                .long("parallel")
                .short("p")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("save-baseline")
                .long("save-baseline")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("baseline")
                .long("baseline")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("nsamples")
                .long("nsamples")
                .takes_value(true)
                .required(false),
        )
        .arg(Arg::with_name("FILTER").required(false))
        .get_matches();

    let mut crit: Criterion = Criterion::default().warm_up_time(Duration::from_millis(500));
    if let Some(nsamples) = matches.value_of("nsamples") {
        crit = crit.sample_size(nsamples.parse::<usize>().unwrap());
    } else {
        crit = crit.sample_size(10);
    }

    if let Some(out) = matches.value_of("output-dir") {
        crit = crit.output_directory(&PathBuf::from(out));
    }

    if let Some(baseline) = matches.value_of("save-baseline") {
        crit = crit.save_baseline(baseline.to_string());
    } else if let Some(baseline) = matches.value_of("baseline") {
        crit = crit.retain_baseline(baseline.to_string());
    }

    if let Some(filter) = matches.value_of("FILTER") {
        crit = crit.with_filter(String::from(filter))
    }

    let data_dir: PathBuf = if let Some(dir) = matches.value_of("data") {
        PathBuf::from(dir)
    } else {
        PathBuf::from("data")
    };
    let queries_dir: PathBuf = if let Some(dir) = matches.value_of("queries") {
        PathBuf::from(dir)
    } else {
        PathBuf::from("queries")
    };

    let use_parallel_joins = matches.is_present("parallel");

    let benches = create_query_input(&data_dir, &queries_dir, use_parallel_joins);

    crit.with_plots()
        .bench_function_over_inputs(
            "count",
            |b: &mut Bencher, obj: &CountBench| {
                for c in obj.def.corpus.iter() {
                    // TODO: preloading all corpora is necessary, but how do we prevent unloading?
                    obj.cs.preload(c).unwrap();
                }
                b.iter(|| {
                    let mut all_corpora_count = 0;
                    for c in obj.def.corpus.iter() {
                        if let Ok(count) = obj.cs.count(c, &obj.def.aql, QueryLanguage::AQL) {
                            all_corpora_count += count;
                        }
                    }
                    assert_eq!(obj.def.count, all_corpora_count);
                });
            },
            benches,
        )
        .final_summary();
}
