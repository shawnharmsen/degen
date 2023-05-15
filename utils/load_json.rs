use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct Test {
  eth_address: String,
  name: String,
  source: String,
}

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
  let client = Client::with_uri_str("mongodb://localhost:27017").await?;
  let collection: Collection<Test> = client.database("test").collection("collection");

  let data = fs::read_to_string("test.json").expect("Unable to read file");

  let v: Vec<HashMap<String, Vec<Test>>> =
    serde_json::from_str(&data).expect("Unable to parse json");

  for map in v {
    for (key, value) in map {
      for item in value {
        let document = Test {
          eth_address: key.clone(),
          name: item.name,
          source: item.source,
        };
        collection.insert_one(document, None).await?;
      }
    }
  }

  Ok(())
}
