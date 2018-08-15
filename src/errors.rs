use Component;
use query::conjunction;
use StringID;

error_chain! {

    foreign_links {
        CSV(::csv::Error);
        IO(::std::io::Error);
        ParseIntError(::std::num::ParseIntError);
        Bincode(::bincode::Error);
        Fmt(::std::fmt::Error);
        Strum(::strum::ParseError);
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

        ImpossibleSearch(reason : String) {
            description("Impossible search expression detected"),
            display("Impossible search expression detected: {}", reason),
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

        AQLSemanticError(desc : String) {
            description("Semantic error in an AQL query"),
            display("AQL semantic error: {}", desc),
        }
    }
}