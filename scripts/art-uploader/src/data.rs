use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct UploaderData {
    pub deviantart_client_id: String,
    pub deviantart_client_secret: String,
    pub deviantart_access_token: Option<String>,
    pub deviantart_refresh_token: Option<String>,
    pub deviantart_access_token_expired_on: Option<DateTime<Utc>>,
}
