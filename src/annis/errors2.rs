use std::error::Error;

pub struct LoadingGraphFailed {
    name: String,
}

impl LoadingGraphFailed {
    pub fn new(name : &str) -> LoadingGraphFailed {
        LoadingGraphFailed {
            name: name.to_string(),
        }
    }
}

impl Error for LoadingGraphFailed {
    fn description(&self) -> &str { format!("Could not load graph {} from disk", &self.name)}
    fn source(&self) -> Option<&(Error + 'static)> { None }
}