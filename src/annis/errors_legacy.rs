#![allow(deprecated)]

error_chain! {

    foreign_links {
        CSV(::csv::Error);
        IO(::std::io::Error);
        ParseIntError(::std::num::ParseIntError);
        Bincode(::bincode::Error);
        Fmt(::std::fmt::Error);
        Strum(::strum::ParseError);
        Regex(::regex::Error);
    }

    errors {
        ImpossibleSearch(reason : String) {
            description("Impossible search expression detected"),
            display("Impossible search expression detected: {}", reason),
        }

        NoSuchString(val : String) {
            description("String does not exist"),
            display("String '{}' does not exist", &val),
        }

        NoSuchCorpus(name : String) {
            description("NoSuchCorpus"),
            display("Corpus {} not found", &name)
        }
    }
}
