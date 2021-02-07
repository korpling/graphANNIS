extern crate clap;
extern crate criterion;

extern crate graphannis;

use clap::*;
use criterion::BenchmarkGroup;
use criterion::{measurement::Measurement, Criterion};
use std::path::{Path, PathBuf};
use std::time::Duration;

use std::sync::Arc;

use graphannis::corpusstorage::{QueryLanguage, SearchQuery};
use graphannis::util;
use graphannis::CorpusStorage;

pub fn create_query_input<M>(
    data_dir: &Path,
    queries_file: &Path,
    use_parallel_joins: bool,
    benchmark_group: &mut BenchmarkGroup<M>,
) where
    M: Measurement,
{
    let cs = Arc::new(CorpusStorage::with_auto_cache_size(data_dir, use_parallel_joins).unwrap());

    let queries = util::get_queries_from_csv(queries_file, true);
    for def in queries.into_iter() {
        let mut bench_name = def.corpus[0].clone();
        bench_name.push_str("/");
        bench_name.push_str(&def.name);

        let cs = cs.clone();

        benchmark_group.bench_function(&bench_name, move |b| {
            for c in def.corpus.iter() {
                // TODO: preloading all corpora is necessary, but how do we prevent unloading?
                cs.preload(c).unwrap();
            }
            let cs = cs.clone();
            let def = def.clone();
            b.iter(move || {
                let search_query = SearchQuery {
                    query: &def.aql,
                    corpus_names: &def.corpus,
                    query_language: QueryLanguage::AQL,
                    timeout: None,
                };
                let count = if let Ok(count) = cs.count(search_query) {
                    count
                } else {
                    0
                };
                assert_eq!(def.count, count);
            });
        });
    }
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

    let mut crit = crit.with_plots();
    let mut group = crit.benchmark_group("count");

    create_query_input(&data_dir, &queries_dir, use_parallel_joins, &mut group);
    group.finish();
}
