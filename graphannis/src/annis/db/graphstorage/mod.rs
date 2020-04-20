
use std::path::Path;
use serde::{Deserialize, Serialize};

pub mod adjacencylist;
pub mod dense_adjacency;
pub mod disk_adjacency;
pub mod linear;
pub mod prepost;
pub mod registry;
pub mod union;

pub fn default_serialize_gs<GS>(gs: &GS, location: &Path) -> Result<()>
where
    GS: Serialize,
{
    let data_path = location.join("component.bin");
    let f_data = std::fs::File::create(&data_path)?;
    let mut writer = std::io::BufWriter::new(f_data);
    bincode::serialize_into(&mut writer, gs)?;
    Ok(())
}

pub fn default_deserialize_gs<GS>(location: &Path) -> Result<GS>
where
    for<'de> GS: std::marker::Sized + Deserialize<'de>,
{
    let data_path = location.join("component.bin");
    let f_data = std::fs::File::open(data_path)?;
    let input = std::io::BufReader::new(f_data);

    let result = bincode::deserialize_from(input)?;

    Ok(result)
}