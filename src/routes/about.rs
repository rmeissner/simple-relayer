use crate::utils::context::Context;
use crate::services::about;
use rocket::response::content::Json;
use anyhow::Result;

#[get("/about")]
pub fn info(context: Context) -> Result<Json<String>> {
    Ok(Json(serde_json::to_string(&about::get_about()?)?))
}