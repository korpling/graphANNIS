// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

use Component;
use query::conjunction;
use graphstorage::registry;
use StringID;

error_chain! {

    foreign_links {
        CSV(::csv::Error);
        IO(::std::io::Error);
        ParseIntError(::std::num::ParseIntError);
        RegistryError(registry::RegistryError);
        Bincode(::bincode::Error);
        Fmt(::std::fmt::Error);
    }

    errors {
        LoadingComponentFailed(c: Component) {
            description("Could not load component from disk"),
            display("Could not load component {} from disk", c),
        }

        LoadingDBFailed(db : String) {
            description("Could not load GraphDB from disk"),
            display("Could not load GraphDB {} from disk", &db),
        }

        ImpossibleSearch(reasons : Vec<conjunction::Error>) {
            description("Impossible search expression detected"),
            display("Impossible search expression detected, reasons: {:?}", reasons),
        }

        NoSuchStringID(id : StringID) {
            description("String ID does not exist"),
            display("String with ID {} does not exist", id),
        }

        NoSuchString(val : String) {
            description("String does not exist"),
            display("String '{}' does not exist", &val),
        }

        NoSuchCorpus(name : String) {
            description("No such corpus found"),
            display("Corpus {} not found", &name)
        }
    }
}