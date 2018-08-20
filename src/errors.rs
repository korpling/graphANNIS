use Component;
use StringID;
use LineColumnRange;

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
            description("NoSuchCorpus"),
            display("Corpus {} not found", &name)
        }

        AQLSyntaxError(short_desc : String, location : Option<LineColumnRange>, hint : Option<String>) {
            description("AQLSyntaxError"),
            display("{}", {
                let mut result = String::new();
                result.push_str(short_desc);
                result.push('\n');
                if let Some(location) = location {
                    result.push_str(&format!("[{}]\n", location));
                }
                if let Some(hint) = hint {
                    result.push_str(&format!("{}\n", hint));
                }
                result
            }),
        }

        AQLSemanticError(short_desc : String, location : Option<LineColumnRange>, hint : Option<String>) {
            description("AQLSemanticError"),
            display("{}", {
                let mut result = String::new();
                result.push_str(short_desc);
                result.push('\n');
                if let Some(location) = location {
                    result.push_str(&format!("[{}]\n", location));
                }
                if let Some(hint) = hint {
                    result.push_str(&format!("{}\n", hint));
                }
                result
            }),
        }
    }
}