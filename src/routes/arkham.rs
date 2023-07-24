use axum::{extract::Path, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, error, info};

use crate::errors::Error;

pub fn create_route() -> Router {
  info!("Creating /arkham/:address route");
  Router::new().route("/arkham/:address", get(query_arkham))
}

async fn query_arkham(Path(address): Path<String>) -> Result<Json<ArkhamResponse>, Error> {
  info!("Querying arkham with address: {}", &address);
  let arkham_api_key = env::var("ARKHAM_API_KEY").expect("ARKHAM_API_KEY must be set");
  let client = reqwest::Client::new();
  let res = client
    .get(format!(
      "https://api.arkhamintelligence.com/intelligence/address/{}/all",
      address
    ))
    .header("API-Key", arkham_api_key)
    .send()
    .await?;

  debug!("Received response with status: {}", res.status());

  if res.status().is_success() {
    let arkham_data: ArkhamResponse = res.json().await?;
    info!("Successfully retrieved Arkham data");
    Ok(Json(arkham_data))
  } else {
    let status = res.status();
    let body = res
      .text()
      .await
      .unwrap_or_else(|_| String::from("Could not retrieve response body"));
    error!("Received a {} error: {}", status, body);
    Err(Error::General(format!(
      "Received a {} error: {}",
      status, body
    )))
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct ArkhamResponse {
  #[serde(rename = "bsc")]
  bsc: ArkhamChainData,
  #[serde(rename = "ethereum")]
  ethereum: ArkhamChainData,
  #[serde(rename = "polygon")]
  polygon: ArkhamChainData,
  #[serde(rename = "arbitrum_one")]
  arbitrum_one: ArkhamChainData,
  #[serde(rename = "avalanche")]
  avalanche: ArkhamChainData,
  #[serde(rename = "optimism")]
  optimism: ArkhamChainData,
}

#[derive(Serialize, Deserialize, Debug)]
struct ArkhamChainData {
  address: Option<String>,
  chain: Option<String>,
  #[serde(rename = "arkhamEntity")]
  arkham_entity: Option<ArkhamEntity>,
  #[serde(rename = "arkhamLabel")]
  arkham_label: Option<ArkhamLabel>,
  #[serde(rename = "isUserAddress")]
  is_user_address: Option<bool>,
  contract: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ArkhamEntity {
  name: Option<String>,
  note: Option<String>,
  id: Option<String>,
  #[serde(rename = "type")]
  entity_type: Option<String>,
  service: Option<String>,
  addresses: Option<Vec<String>>,
  website: Option<String>,
  twitter: Option<String>,
  crunchbase: Option<String>,
  linkedin: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ArkhamLabel {
  name: Option<String>,
  address: Option<String>,
  #[serde(rename = "chainType")]
  chain_type: Option<String>,
}
