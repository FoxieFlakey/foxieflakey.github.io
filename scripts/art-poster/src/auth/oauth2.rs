use std::{collections::HashMap, marker::PhantomData, sync::LazyLock, time::Duration};

use anyhow::{Context, anyhow, bail};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use rand::{Rng, rngs::StdRng};
use regex::{Regex, regex};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;

use crate::{auth::oauth2_providers::Provider, ui};

// Uses reference at https://www.rfc-editor.org/info/rfc6749/

#[derive(Serialize, Deserialize)]
pub struct OAuth2State<P: Provider> {
    client_id: String,
    refresh_token: Option<String>,
    client_secret: String,
    // .0 the token
    // .1 expiry date
    access_token: Option<(String, DateTime<Utc>)>,
    #[serde(default, skip)]
    reqwest: reqwest::Client,
    _phantom: PhantomData<P>,
}

static VALID_CLIENT_ID: LazyLock<&Regex> = LazyLock::new(|| regex!(r"^[\x20-\x7e]*$"));
static VALID_CLIENT_SECRET: LazyLock<&Regex> = LazyLock::new(|| regex!(r"^[\x20-\x7e]*$"));

static REDIRECT_URL: &str = "http://127.0.0.1:8080";

impl<P: Provider> OAuth2State<P> {
    pub async fn new_interactive() -> anyhow::Result<Self> {
        println!("Configuring new {} account", P::name());

        let client_id = ui::ask_question(
            "Client ID",
            false,
            false,
            None,
            Some((
                &VALID_CLIENT_ID,
                "Only printable ASCII character allowed in client ID",
            )),
            None,
        )
        .await
        .context("Asking client id")?
        .context("Client ID is required")?;

        let client_secret = ui::ask_question(
            "Client secret",
            false,
            false,
            None,
            Some((
                &VALID_CLIENT_SECRET,
                "Only printable ASCII character allowed in client ID",
            )),
            None,
        )
        .await
        .context("Asking client secret")?
        .context("Client secret is required")?;

        Ok(Self {
            client_id,
            client_secret,
            refresh_token: None,
            access_token: None,
            reqwest: Default::default(),
            _phantom: PhantomData,
        })
    }

    // Returns access token
    async fn authorize<'a>(&'a mut self) -> anyhow::Result<&'a str> {
        let state = random_bytes();

        // 4.1.1 Authorization Request
        let mut url = P::get_authorize_url();
        let scopes = P::scope().join(" ");
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", REDIRECT_URL)
            .append_pair("state", &state)
            .append_pair("scope", &scopes);

        // Extension for PKCE protection, I made it mandataory to use this module :3
        // so no need ot handle legacy stuffs bwehs

        // RFC 7636 4.1 Create code verifier
        let code_verifier = random_bytes();

        // RFC 7636 4.2 Create challenege
        let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(&code_verifier));

        // RFC 7636 4.3 Add it to go with authorization request :3 and
        // always use SHA256 method
        url.query_pairs_mut()
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256");

        // Spin HTTP server, so the server is ready to listen BEFORE user can see the URL
        let server = tiny_http::Server::http("127.0.0.1:8080")
            .map_err(anyhow::Error::from_boxed)
            .context("Starting HTTP server for receiving authorization response")?;

        // Then ask user to open
        let encoded = url.to_string();
        println!("Please open this to authorize this app");
        println!("{encoded}");

        // 4.1.2 Authorization response, here just reads requests
        let mut auth_code = None;
        'serving_loop: for request in server.incoming_requests() {
            let url = match Url::parse(&format!("{REDIRECT_URL}/{}", request.url())) {
                Ok(url) => url,
                Err(e) => {
                    request
                    .respond(tiny_http::Response::from_string(format!(
                        "Your client sent invalid URL's path component (after {REDIRECT_URL}): {e}",
                    )))
                    .context("Cannot respond to client")?;
                    continue;
                }
            };

            let mut map = HashMap::new();
            for (key, value) in url.query_pairs() {
                if map.contains_key(&key) {
                    request
                        .respond(tiny_http::Response::from_string(format!(
                            "Your browser or authorization server sent duplicated query key '{key}' more than once. You can retry",
                        )))
                        .context("Cannot respond to client")?;
                    continue 'serving_loop;
                }
                map.insert(key, value);
            }
            let Some(response_state) = map.get("state") else {
                request
                    .respond(tiny_http::Response::from_string(format!(
                        "Your browser or authorization server didn't send the 'state'. You can retry",
                    )))
                    .context("Cannot respond to client")?;
                continue;
            };

            if response_state != &state {
                request
                    .respond(tiny_http::Response::from_string(format!(
                        "Your browser or authorization server send invalid 'state' expected {state} got {response_state}",
                    )))
                    .context("Cannot respond to client")?;
                continue;
            }

            // Check if its error for section 4.1.2.1 error response
            if let Some(code) = map.get("error") {
                let desc = map.get("error_description");
                let uri = map.get("error_uri");
                let error_message = format!(
                    "An error from authorization server (code: {code:?}, uri: {uri:?}): {desc:?}"
                );
                request
                    .respond(tiny_http::Response::from_string(format!(
                        "Got an error from authorization server: {error_message}\nDo not retry the program no longer listening. You can close this tab",
                    )))
                    .context("Cannot respond to client")?;
                bail!("{error_message}");
            }

            // It is ok response for section 4.1.2 authorization response
            let Some(code) = map.get("code") else {
                request
                    .respond(tiny_http::Response::from_string(format!(
                        "Authorization server or your browser did not send the authorization code back. You can retry",
                    )))
                    .context("Cannot respond to client")?;
                continue;
            };

            request
                .respond(tiny_http::Response::from_string(format!(
                    "You can close this tab now",
                )))
                .context("Cannot respond to client")?;
            auth_code = Some(code.to_string());
            break;
        }

        let Some(code) = auth_code else {
            bail!("Cannot get authorization code");
        };

        self.get_tokens(GrantData::AuthorizationCode {
            code,
            code_verifier,
        })
        .await
        .context("Getting tokens from authorization code")
    }

    async fn get_tokens<'a>(&'a mut self, grant_type: GrantData) -> Result<&'a str, GetTokenError> {
        let mut url = P::get_token_url();
        match grant_type {
            GrantData::AuthorizationCode {
                code,
                code_verifier,
            } => {
                url.query_pairs_mut()
                    .append_pair("grant_type", "authorization_code")
                    .append_pair("code", &code)
                    .append_pair("redirect_uri", REDIRECT_URL)
                    //     // RFC 7636 4.5, can now put the code verifier here
                    .append_pair("code_verifier", &code_verifier);
            }
            GrantData::RefreshToken(token) => {
                url.query_pairs_mut()
                    .append_pair("grant_type", "refresh_token")
                    .append_pair("refresh_token", &token)
                    .append_pair("scope", &P::scope().join(" "));
            }
        };

        // Section 2.3 The client also authenticates and HTTP password method is only required one, so lets use that
        // each of id and secret is encoded with URL encoding. The client id become username and secret become password part
        //
        // Its used HTTP Basic authorization scheme
        let encoded_client_id = urlencoding::encode(&self.client_id);
        let encoded_client_secret = urlencoding::encode(&self.client_secret);

        let req = self
            .reqwest
            .post(url)
            .basic_auth(encoded_client_id, Some(encoded_client_secret));

        // Send the request
        let response = req.send().await.context("Sending token request")?;

        let status = response.status();
        if status != StatusCode::BAD_REQUEST && status != StatusCode::OK {
            return Err(anyhow!("Unknown status code from server: {status}").into());
        }

        let response_bytes = response
            .bytes()
            .await
            .context("Getting response bytes from token endpoint")?;

        if status == StatusCode::BAD_REQUEST {
            // Error from server, section 5.2

            #[derive(Deserialize)]
            struct ErrorResponse {
                error: String,
                error_description: String,
                error_uri: String,
            }

            let ErrorResponse {
                error: code,
                error_description: desc,
                error_uri: uri,
            } = serde_json::from_slice::<ErrorResponse>(&response_bytes)
                .context("Parsing token server response")?;

            match code.as_str() {
                "invalid_grant" => return Err(GetTokenError::InvalidGrantData),
                x => {
                    return Err(anyhow!(
                        "An error from token server (code: {x:?}, uri: {uri:?}): {desc:?}"
                    )
                    .into());
                }
            }
        }

        // Section 5.1 success response

        #[derive(Deserialize)]
        struct TokenResponse {
            expires_in: Option<u64>,
            access_token: String,
            token_type: String,
            refresh_token: Option<String>,
        }

        let response = serde_json::from_slice::<TokenResponse>(&response_bytes)
            .context("Parsing server's token endpoint success response")?;

        if response.token_type != "Bearer" {
            return Err(anyhow!(
                "This client don't support non bearer token type. but server returns {}",
                response.token_type
            )
            .into());
        }

        let expiry = Utc::now()
            + Duration::from_secs(
                response
                    .expires_in
                    .unwrap_or_else(P::default_access_token_expire_duration)
                    // Lets give headroom for 300, to notice bit earlier
                    .saturating_sub(300),
            );
        self.access_token = Some((response.access_token, expiry));
        self.refresh_token = response.refresh_token;
        Ok(&self.access_token.as_ref().unwrap().0)
    }

    pub async fn get_access_token<'a>(&'a mut self) -> anyhow::Result<&'a str> {
        if let Some((token, expiry)) = &self.access_token
            && &Utc::now() < expiry
        {
            return Ok(token);
        }

        if let Some(refresh) = self.refresh_token.clone() {
            match self.get_tokens(GrantData::RefreshToken(refresh)).await {
                Ok(access_token) => {
                    return Ok(access_token);
                }
                Err(e) => match e {
                    GetTokenError::InvalidGrantData => {
                        // Refresh token is probably expired lets re authorize
                    }
                    GetTokenError::Other(error) => {
                        return Err(error).context("Refreshing access token");
                    }
                },
            }
        }

        // Refresh token and access token is expired here. Lets reauthorize
        self.authorize()
            .await
            .context("Authorizing to get new access token and maybe refresh token")
    }
}

#[derive(Error, Debug)]
enum GetTokenError {
    #[error(
        "Invalid grant data (either invalid authorization code or expired/invalid refresh token)"
    )]
    InvalidGrantData,
    #[error("Other errors")]
    Other(
        #[source]
        #[from]
        anyhow::Error,
    ),
}

enum GrantData {
    AuthorizationCode { code: String, code_verifier: String },
    RefreshToken(String),
}

// Its just hex which is 0-9 and A-F valid for 'state' key in
// authorization request and code_verifier part of same request
fn random_bytes() -> String {
    let mut data = [0; 64];
    rand::make_rng::<StdRng>().fill_bytes(&mut data);
    hex::encode(data)
}
