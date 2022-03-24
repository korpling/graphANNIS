extern crate clap;
extern crate criterion;

extern crate graphannis;

use clap::*;
use criterion::BenchmarkGroup;
use criterion::{measurement::Measurement, Criterion};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use std::sync::Arc;

use graphannis::corpusstorage::{QueryLanguage, SearchQuery};
use graphannis::util::{self, SearchDef};
use graphannis::CorpusStorage;

pub fn create_query_input<M>(
    data_dir: &Path,
    queries: Vec<SearchDef>,
    use_parallel_joins: bool,
    benchmark_group: &mut BenchmarkGroup<M>,
) where
    M: Measurement,
{
    let cs = Arc::new(CorpusStorage::with_auto_cache_size(data_dir, use_parallel_joins).unwrap());

    for def in queries.into_iter() {
        let bench_name = def.name.clone();
        let cs = cs.clone();

        benchmark_group.bench_function(&bench_name, move |b| {
            for c in def.corpus.iter() {
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
            Arg::with_name("data")
                .long("data")
                .short("d")
                .takes_value(true)
                .required(true)
                .help("Path to the data directory to use"),
        )
        .arg(
            Arg::with_name("queries")
                .long("queries")
                .short("q")
                .takes_value(true)
                .required(true)
                .help("Path to the CSV file with the queries to benchmark (with the columns \"name\", \"aql\", \"corpus\", \"count\")"),
        )
        .arg(
            Arg::with_name("parallel")
                .long("parallel")
                .short("p")
                .takes_value(false)
                .required(false)
                .help("Use parallel joins when possible"),
        )
        .arg(
            Arg::with_name("save-baseline")
                .long("save-baseline")
                .takes_value(true)
                .required(false)
                .help("Save results under a named baseline"),
        )
        .arg(
            Arg::with_name("baseline")
                .long("baseline")
                .takes_value(true)
                .required(false)
                .conflicts_with("save-baseline")
                .help("Compare to a named baseline"),
        )
        .arg(
            Arg::with_name("sample-size")
                .long("sample-size")
                .takes_value(true)
                .required(false)
                .help("Changes the default size of the sample for this run. [default: 10]"),
        )
        .arg(Arg::with_name("FILTER").required(false))
        .arg(Arg::with_name("measurement-time")
        .long("measurement-time")
        .takes_value(true)
        .help(&format!("Changes the default measurement time for this run. [default: 5]")))
        .get_matches();

    let mut crit: Criterion = Criterion::default().warm_up_time(Duration::from_millis(500));
    if let Some(nsamples) = matches.value_of("sample-size") {
        crit = crit.sample_size(nsamples.parse::<usize>().unwrap());
    } else {
        crit = crit.sample_size(10);
    }

    if let Some(baseline) = matches.value_of("save-baseline") {
        crit = crit.save_baseline(baseline.to_string());
    } else if let Some(baseline) = matches.value_of("baseline") {
        crit = crit.retain_baseline(baseline.to_string());
    }

    if matches.is_present("measurement-time") {
        let t = value_t_or_exit!(matches.value_of("measurement-time"), u64);
        crit = crit.measurement_time(Duration::from_secs(t));
    }

    if let Some(filter) = matches.value_of("FILTER") {
        crit = crit.with_filter(String::from(filter))
    }

    let data_dir: PathBuf = if let Some(dir) = matches.value_of("data") {
        PathBuf::from(dir)
    } else {
        PathBuf::from("data")
    };
    let queries_file: PathBuf = if let Some(dir) = matches.value_of("queries") {
        PathBuf::from(dir)
    } else {
        PathBuf::from("queries")
    };

    let use_parallel_joins = matches.is_present("parallel");

    let mut crit = crit.with_plots().with_output_color(true);

    let queries = util::get_queries_from_csv(&queries_file, true);
    // Create a benchmark group for each corpus
    let used_corpora: BTreeSet<String> = queries
        .iter()
        .filter_map(|def| def.corpus.first().map(|c| c.to_string()))
        .collect();

    for c in used_corpora {
        let mut group = crit.benchmark_group(&c);
        let queries_for_corpus: Vec<_> = queries
            .iter()
            .filter(|def| def.corpus.first() == Some(&c))
            .cloned()
            .collect();
        create_query_input(
            &data_dir,
            queries_for_corpus,
            use_parallel_joins,
            &mut group,
        );
        group.finish();
    }

    crit.final_summary();
}
