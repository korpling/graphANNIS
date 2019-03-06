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
    }
}
