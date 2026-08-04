use std::collections::HashMap;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{TimeDelta, Utc};
use rand::Rng;
use reqwest::multipart::{self, Form};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tiny_http::{Response, Server};
use url::Url;

use crate::{Args, data::UploaderData};

pub fn upload(data: &mut UploaderData, args: &Args, art_data: &[u8]) -> Result<(), String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(upload_impl(data, args, art_data))
}

async fn upload_impl(data: &mut UploaderData, args: &Args, art_data: &[u8]) -> Result<(), String> {
    let mut http_client = reqwest::ClientBuilder::new().build().unwrap();
    if
    // Its expired
    (data.deviantart_access_token_expired_on.is_some() && Utc::now() > data.deviantart_access_token_expired_on.unwrap()) ||
        // Assume expired if expiry key missing
        data.deviantart_access_token_expired_on.is_none() ||
        // Assume expired if refresh token missing
        data.deviantart_refresh_token.is_none()
    {
        // Token expired
        data.deviantart_access_token = None;
        data.deviantart_access_token_expired_on = None;
        data.deviantart_refresh_token = None;
    }

    if data.deviantart_access_token.is_none() {
        data.deviantart_access_token = Some(
            get_access_token(
                &mut http_client,
                data,
                data.deviantart_refresh_token.clone(),
            )
            .await?,
        );
    }
    let access_token = data.deviantart_access_token.as_ref().unwrap();
    
    let mut form = Form::new()
        .text("title", args.title.to_string())
        .text("artist_comments", args.description.to_string())
        .text("noai", "true")
        .text("is_ai_generated", "false").part(
            "file",
            multipart::Part::bytes(art_data.to_vec())
                .file_name(args.filename.to_string())
        );
    for val in &args.keywords {
        form = form.text("tags", val.to_string());
    }
    
    let response = http_client.post("https://www.deviantart.com/api/v1/oauth2/stash/submit")
        .bearer_auth(access_token)
        .header("dA-minor-version", "20240701")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Cannot send request to submit art to sta.sh: {e}"))?;
    let is_success = response.status().is_success();
    let response_bytes = response.bytes()
        .await
        .map_err(|e| format!("Cannot read bytes when submiting art to sta.sh: {e}"))?;
    
    if !is_success {
        #[derive(Deserialize)]
        struct ErrorResponse {
            error: String,
            error_description: String,
        }

        let response = serde_json::from_slice::<ErrorResponse>(&response_bytes)
            .map_err(|x| format!("Cannot parse error response body for submiting to sta.sh: {x}"))?;

        return Err(format!(
            "Cannot submit art to sta.sh: {}: {}",
            response.error, response.error_description
        ));
    }
    
    #[derive(Deserialize)]
    pub struct SubmitSuccessResponse {
        status: String,
        itemid: u64
    }
    let response = serde_json::from_slice::<SubmitSuccessResponse>(&response_bytes)
        .map_err(|x| format!("Cannot parse success response body for submiting to sta.sh: {x}"))?;
    
    if response.status != "success" {
        return Err(format!("Expecting 'success' response from submit got '{}'", response.status));
    }
    
    let item_id = response.itemid;
    
    let mut form = Form::new()
        .text("is_mature", "false")
        .text("allow_comments", "true")
        .text("add_watermark", "false")
        .text("display_resolution", "0")
        .text("allow_free_download", "true")
        .text("noai", "true")
        .text("is_ai_generated", "false")
        .text("itemid", item_id.to_string());
    
    for val in &args.keywords {
        form = form.text("tags", val.to_string());
    }
    
    let response = http_client.post("https://www.deviantart.com/api/v1/oauth2/stash/publish")
        .bearer_auth(access_token)
        .header("dA-minor-version", "20240701")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Cannot send request to publish on DeviantArt: {e}"))?;
    
    let is_success = response.status().is_success();
    let response_bytes = response.bytes()
        .await
        .map_err(|e| format!("Cannot read bytes when publishing art to sta.sh: {e}"))?;
    
    if !is_success {
        #[derive(Deserialize)]
        struct ErrorResponse {
            error: String,
            error_description: String,
        }

        let response = serde_json::from_slice::<ErrorResponse>(&response_bytes)
            .map_err(|x| format!("Cannot parse error response body for publishing to sta.sh: {x}"))?;

        return Err(format!(
            "Cannot submit art to sta.sh: {}: {}",
            response.error, response.error_description
        ));
    }
    
    #[derive(Deserialize)]
    pub struct PublishSuccessResponse {
        status: String,
        url: String,
        #[expect(unused)]
        deviationid: String
    }
    let response = serde_json::from_slice::<PublishSuccessResponse>(&response_bytes)
        .map_err(|x| format!("Cannot parse success response body for submiting to sta.sh: {x}"))?;
    
    if response.status != "success" {
        return Err(format!("Expecting 'success' response from submit got '{}'", response.status));
    }
    
    println!("Publish at {}", response.url);
    
    Ok(())
}

// https://deviantart.readme.io/docs/authentication
// refresh is some to refresh token
async fn get_access_token(
    http_client: &mut reqwest::Client,
    data: &mut UploaderData,
    refresh: Option<String>,
) -> Result<String, String> {
    let client_id = &data.deviantart_client_id;
    let client_secret = &data.deviantart_client_secret;

    let redirect_uri = "http://localhost:8080";
    let (code_verifier, challenge) = generate_pkce();
    let generated_state = generate_oauth_state();
    let authorize_url = format!(
        "https://www.deviantart.com/oauth2/authorize?response_type=code&client_id={client_id}&redirect_uri={redirect_uri}&scope=basic stash&state={generated_state}&code_challenge={challenge}&code_challenge_method=S256",
    ).replace(' ', "%20");

    // Gathering token
    let token_endpoint = "https://www.deviantart.com/oauth2/token";
    let mut token_url = Url::parse(token_endpoint).unwrap();
    let mut queries = token_url.query_pairs_mut();

    // Common stuffs
    queries
        .append_pair("client_id", client_id)
        .append_pair("client_secret", client_secret);
    
    if let Some(refresh_token) = refresh {
        // Refreshing existing token
        queries.append_pair("grant_type", "refresh_token")
            .append_pair("refresh_token", &refresh_token);
    } else {
        let server = Server::http("127.0.0.1:8080").unwrap();
        println!("Please open this to authorize for uploading (access token is not saved)");
        println!("{authorize_url}");

        let mut auth_code = None;
        for request in server.incoming_requests() {
            let url_parsed = Url::parse(&format!("http://localhost/{}", request.url())).unwrap();
            let query_map = url_parsed
                .query_pairs()
                .collect::<HashMap<_, _>>();
            if let Some(val) = query_map.get("code") {
                let code = val.to_string();
                if let Some(val) = query_map.get("state") {
                    request
                        .respond(Response::from_string("You can close this tab Uwu"))
                        .unwrap();
                    auth_code = Some((code, val.to_string()));
                    break;
                }
            }

            if let Some(error_code) = query_map.get("error") {
                let error_desc = query_map
                    .get("error_description")
                    .map(|x| x.to_string())
                    .unwrap();
                let response = format!(
                    "Cannot get authorization code: {}: {}",
                    error_code, error_desc
                );
                request.respond(Response::from_string(&response)).unwrap();
                return Err(response);
            }

            // Send the response back to the client
            // we dont care if it failed lol
            request
                .respond(Response::from_string("idk this URL. please retry? maybe or report :<"))
                .unwrap();
        }

        let Some((auth_code, state)) = auth_code else {
            return Err("Cannot get authoriztion code, try again".to_string());
        };

        if state != generated_state {
            return Err(
                "State parameter on return URL is not the same one as authorize URL".to_string(),
            );
        }

        queries
            .append_pair("grant_type", "authorization_code")
            .append_pair("code", &auth_code)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("code_verifier", &code_verifier);
    }
    drop(queries);

    let response = http_client
        .post(token_url)
        .send()
        .await
        .map_err(|x| format!("Cannot request at {token_endpoint}: {x}"))?;
    let is_success = response.status().is_success();
    let response_bytes = response
        .bytes()
        .await
        .map_err(|x| format!("Cannot get response body for {token_endpoint}: {x}"))?;

    if !is_success {
        #[derive(Deserialize)]
        struct ErrorResponse {
            error: String,
            error_description: String,
        }

        let response = serde_json::from_slice::<ErrorResponse>(&response_bytes)
            .map_err(|x| format!("Cannot parse error response body for {token_endpoint}: {x}"))?;

        return Err(format!(
            "Error occured when requesting token: {}: {}",
            response.error, response.error_description
        ));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        expires_in: u64,
        status: String,
        access_token: String,
        token_type: String,
        refresh_token: String,
        #[expect(unused)]
        scope: String,
    }

    let response = serde_json::from_slice::<TokenResponse>(&response_bytes)
        .map_err(|x| format!("Cannot parse response body for {token_endpoint}: {x}"))?;

    if response.token_type != "Bearer" {
        return Err(format!(
            "DeviantArt sent wrong token type, expecting 'Bearer' got {}",
            response.token_type
        ));
    }

    if response.status != "success" {
        return Err(format!(
            "DeviantArt sent wrong status, expecting 'success' got {}",
            response.status
        ));
    }

    data.deviantart_access_token_expired_on =
        Some(Utc::now() + TimeDelta::seconds(i64::try_from(response.expires_in).unwrap() - 60));
    data.deviantart_access_token = Some(response.access_token.clone());
    data.deviantart_refresh_token = Some(response.refresh_token);

    Ok(response.access_token)
}

// Gemini AI slop generated :3
fn generate_oauth_state() -> String {
    // 1. Create a buffer for 32 bytes of cryptographically secure randomness
    let mut random_bytes = [0u8; 32];

    // 2. Fill the buffer using the OS entropy source
    rand::rng().fill_bytes(&mut random_bytes);

    // 3. Encode the bytes into a clean, URL-safe Base64 string
    URL_SAFE_NO_PAD.encode(random_bytes)
}

// Gemini AI slop generated :3
fn generate_pkce() -> (String, String) {
    // 1. Generate 32 bytes of cryptographically secure random data
    let mut random_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut random_bytes);

    // 2. Base64URL-encode the raw bytes to create the Code Verifier
    let code_verifier = URL_SAFE_NO_PAD.encode(random_bytes);

    // 3. Hash the Code Verifier using SHA-256
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash_result = hasher.finalize();

    // 4. Base64URL-encode the hash to create the Code Challenge
    let code_challenge = URL_SAFE_NO_PAD.encode(hash_result);

    (code_verifier, code_challenge)
}
