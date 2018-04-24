#[macro_use]
extern crate bencher;
extern crate rand;
extern crate graphannis;

use bencher::Bencher;
use graphannis::annostorage::AnnoStorage;
use graphannis::{Annotation, AnnoKey, StringID};

use rand::distributions::{IndependentSample, Range};
use rand::seq;

fn retrieve_annos_for_node(bench: &mut Bencher) {


    let mut annos : AnnoStorage<usize> = AnnoStorage::new();

    let mut rng = rand::thread_rng();

    let ns_range : Range<StringID> = Range::new(0,3);
    let name_range : Range<StringID> = Range::new(0,20);
    let val_range : Range<StringID> = Range::new(0,100);

    // insert 10 000 random annotations
    for i in 0..10_000 {
        let a = Annotation {
            key: AnnoKey {
                ns: ns_range.ind_sample(&mut rng),
                name: name_range.ind_sample(&mut rng) ,
            },
            val: val_range.ind_sample(&mut rng),
        };
        annos.insert(i, a);
    }

    // sample 1000 items to get the annotation value from
    let samples : Vec<usize> = seq::sample_indices(&mut rng, 10_000, 1_000);

    bench.iter(move || {
        let mut sum = 0;
        for i in samples.iter() {
            let a = annos.get_all(i);
            sum += a.len();
        }
        assert!(sum > 0);
    })
}


benchmark_group!(annostorage, retrieve_annos_for_node);

benchmark_main!(annostorage);