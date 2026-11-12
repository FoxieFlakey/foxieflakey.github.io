use url::Url;
use url_macro::url;

pub trait Provider {
    fn name() -> &'static str;
    fn scope() -> &'static [&'static str];
    fn get_token_url() -> Url;
    fn get_authorize_url() -> Url;
    fn default_access_token_expire_duration() -> u64;
}

pub struct DeviantArt;

impl Provider for DeviantArt {
    fn get_token_url() -> Url {
        url!("https://www.deviantart.com/oauth2/token")
    }

    fn get_authorize_url() -> Url {
        url!("https://www.deviantart.com/oauth2/authorize")
    }

    fn name() -> &'static str {
        "DeviantArt"
    }

    fn scope() -> &'static [&'static str] {
        static SCOPES: [&str; 2] = ["basic", "stash"];
        &SCOPES
    }

    fn default_access_token_expire_duration() -> u64 {
        // 1 hour
        3600
    }
}
