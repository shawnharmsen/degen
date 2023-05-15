use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Serialize, Deserialize, Debug)]
struct AddressData {
  name: String,
  source: String,
}

type InputData = HashMap<String, Vec<AddressData>>;
type OutputData = HashMap<String, Vec<OutputAddressData>>;

#[derive(Serialize, Deserialize, Debug)]
struct OutputAddressData {
  eth_address: String,
  name: String,
  source: String,
}

fn main() -> std::io::Result<()> {
  // Read the data from the file.
  let file = File::open("test.json")?;
  let reader = BufReader::new(file);
  let data: InputData = serde_json::from_reader(reader).unwrap();

  // Transform the data.
  let transformed: OutputData = data
    .into_iter()
    .map(|(address, mut records)| {
      let new_records: Vec<OutputAddressData> = records
        .drain(..)
        .map(|record| OutputAddressData {
          eth_address: address.clone(),
          name: record.name,
          source: record.source,
        })
        .collect();
      (address, new_records)
    })
    .collect();

  // Write the transformed data to the output file.
  let file = File::create("output.json")?;
  let writer = BufWriter::new(file);
  serde_json::to_writer(writer, &transformed)?;

  Ok(())
}
