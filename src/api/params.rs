use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct SubmissionsParams {
    pub uid: Option<i64>,
}
