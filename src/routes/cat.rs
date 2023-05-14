use axum::http::StatusCode;
use axum::{
  extract::{Path, Query},
  routing::{delete, get, post, put},
  Json, Router,
};
use bson::doc;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::debug;
use wither::mongodb::options::FindOptions;

use crate::errors::Error;
use crate::models::cat::{Cat, PublicCat};
use crate::utils::custom_response::{CustomResponse, CustomResponseBuilder};
use crate::utils::models::ModelExt;
use crate::utils::pagination::Pagination;
use crate::utils::request_query::RequestQuery;
use crate::utils::to_object_id::to_object_id;
use crate::utils::token::TokenUser;

pub fn create_route() -> Router {
  Router::new()
    .route("/cats", post(create_cat))
    .route("/cats", get(query_cats))
    .route("/cats/:id", get(get_cat_by_id))
    .route("/cats/:id", delete(remove_cat_by_id))
    .route("/cats/:id", put(update_cat_by_id))
    .route("/arkham/:address", get(query_arkham)) // Add the new route here
}

async fn create_cat(
  user: TokenUser,
  Json(payload): Json<CreateCat>,
) -> Result<CustomResponse<PublicCat>, Error> {
  let cat = Cat::new(user.id, payload.name);
  let cat = Cat::create(cat).await?;
  let res = PublicCat::from(cat);

  let res = CustomResponseBuilder::new()
    .body(res)
    .status_code(StatusCode::CREATED)
    .build();

  Ok(res)
}

async fn query_cats(
  user: TokenUser,
  Query(query): Query<RequestQuery>,
) -> Result<CustomResponse<Vec<PublicCat>>, Error> {
  let pagination = Pagination::build_from_request_query(query);

  let options = FindOptions::builder()
    .sort(doc! { "created_at": -1_i32 })
    .skip(pagination.offset)
    .limit(pagination.limit as i64)
    .build();

  let (cats, count) = Cat::find_and_count(doc! { "user": &user.id }, options).await?;
  let cats = cats.into_iter().map(Into::into).collect::<Vec<PublicCat>>();

  let res = CustomResponseBuilder::new()
    .body(cats)
    .pagination(pagination.count(count).build())
    .build();

  debug!("Returning cats");
  Ok(res)
}

async fn get_cat_by_id(user: TokenUser, Path(id): Path<String>) -> Result<Json<PublicCat>, Error> {
  let cat_id = to_object_id(id)?;
  let cat = Cat::find_one(doc! { "_id": cat_id, "user": &user.id }, None)
    .await?
    .map(PublicCat::from);

  let cat = match cat {
    Some(cat) => cat,
    None => {
      debug!("Cat not found, returning 404 status code");
      return Err(Error::not_found());
    }
  };

  debug!("Returning cat");
  Ok(Json(cat))
}

async fn remove_cat_by_id(
  user: TokenUser,
  Path(id): Path<String>,
) -> Result<CustomResponse<()>, Error> {
  let cat_id = to_object_id(id)?;
  let delete_result = Cat::delete_one(doc! { "_id": cat_id, "user": &user.id }).await?;

  if delete_result.deleted_count == 0 {
    debug!("Cat not found, returning 404 status code");
    return Err(Error::not_found());
  }

  let res = CustomResponseBuilder::new()
    .status_code(StatusCode::NO_CONTENT)
    .build();

  Ok(res)
}

async fn update_cat_by_id(
  user: TokenUser,
  Path(id): Path<String>,
  Json(payload): Json<UpdateCat>,
) -> Result<Json<PublicCat>, Error> {
  let cat_id = to_object_id(id)?;
  let update = bson::to_document(&payload).unwrap();

  let cat = Cat::find_one_and_update(
    doc! { "_id": &cat_id, "user": &user.id },
    doc! { "$set": update },
  )
  .await?
  .map(PublicCat::from);

  let cat = match cat {
    Some(cat) => cat,
    None => {
      debug!("Cat not found, returning 404 status code");
      return Err(Error::not_found());
    }
  };

  debug!("Returning cat");
  Ok(Json(cat))
}

async fn query_arkham(Path(address): Path<String>) -> Result<Json<ArkhamResponse>, Error> {
  dotenv().ok();
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

  if res.status().is_success() {
    let arkham_data: ArkhamResponse = res.json().await?;
    Ok(Json(arkham_data))
  } else {
    // Log the status code and the response body for more information
    let status = res.status();
    let body = res
      .text()
      .await
      .unwrap_or_else(|_| String::from("Could not retrieve response body"));
    tracing::error!("Received a {} error: {}", status, body);
    Err(Error::General(format!(
      "Received a {} error: {}",
      status, body
    )))
  }
}

#[derive(Deserialize)]
struct CreateCat {
  name: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateCat {
  name: String,
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
