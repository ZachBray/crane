use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::str;
use actix_web::client;
use actix_web::http::header;
use actix_web::http::StatusCode;
use actix_web::HttpMessage;
use chrono::prelude::*;
use chrono::Duration;
use fnv::FnvHashMap;
use futures::prelude::*;
use futures::future::ok;
use futures::future::err;
use jsonwebtoken::encode;
use jsonwebtoken::Header;
use jsonwebtoken::Algorithm;
use serde_json;
use error;
use model::values::installation_id::InstallationId;
use github_client::github_client::GitHubClient;
use github_client::commands::store_token::StoreToken;

struct JwtState {
    token: Arc<String>,
    expiry: DateTime<Utc>,
}

impl JwtState {
    fn generate(key: &[u8]) -> Self {
        #[derive(Serialize)]
        struct Claims {
            iat: i64,
            exp: i64,
            iss: &'static str,
        }

        let now = Utc::now();
        let expiry = now + Duration::seconds(60 * 10);
        let payload = Claims {
            iat: now.timestamp(),
            exp: expiry.timestamp(),
            iss: "15983", // TODO get from config
        };
        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &payload, key)
            .expect("Failed to encode JWT");
        JwtState {
            token: Arc::new(token),
            expiry,
        }
    }

    fn is_close_to_expiry(&self) -> bool {
        Utc::now() + Duration::seconds(60) > self.expiry
    }
}

#[derive(Deserialize, Clone)]
pub struct AccessToken {
    token: Arc<String>,
    #[serde(rename = "expires_at")]
    expiry: DateTime<Utc>,
}

impl AccessToken {
    fn request(jwt: &str, installation_id: InstallationId)
               -> impl Future<Item=AccessToken, Error=error::Error>
    {
        info!("Requesting installation token for installation: {}", installation_id.0);
        let url = format!("https://api.github.com/installations/{}/access_tokens",
                          installation_id.0);
        let auth_header = format!("Bearer {}", jwt);
        info!("Header: {}", auth_header);
        client::post(url)
            .header(header::ACCEPT, "application/vnd.github.machine-man-preview+json")
            .header(header::AUTHORIZATION, auth_header)
            .finish()
            .unwrap()
            .send()
            .from_err::<error::Error>()
            .inspect(|response| {
                info!("Received response for installation token creation with status: {:?}", response.status());
            })
            .and_then(|response|
                response.body()
                    .limit(10 * 1024)
                    .from_err::<error::Error>()
                    .and_then(|body| {
                        let body_str = str::from_utf8(body.as_ref()).unwrap_or("");
                        info!("Received: {}", body_str);
                        serde_json::from_slice::<AccessToken>(body.as_ref())
                            .map_err(|e| error::Error::ParseError(e))
                    }))
    }

    fn is_close_to_expiry(&self) -> bool {
        Utc::now() + Duration::seconds(60) > self.expiry
    }
}

pub struct GitHubClientState {
    private_key: Vec<u8>,
    jwt: Option<JwtState>,
    pub access_tokens: FnvHashMap<InstallationId, AccessToken>,
}

impl GitHubClientState {
    pub fn new(der_file: String) -> Result<Self, error::Error> {
        let private_key = GitHubClientState::read_to_string(der_file)?;
        if private_key.len() == 0 {
            Err(error::Error::PrivateKeyLoadError)
        } else {
            Ok(GitHubClientState {
                private_key,
                jwt: None,
                access_tokens: FnvHashMap::default(),
            })
        }
    }

    pub fn installation_token<'a>(&'a mut self,
                                  mail_box: &'a GitHubClient,
                                  installation_id: InstallationId)
                                  -> impl Future<Item=Arc<String>, Error=error::Error> {
        let existing_token =
            match self.access_tokens.remove(&installation_id) {
                Some(ref token) if !token.is_close_to_expiry() => ok(token.token.clone()),
                Some(_) | None => err(())
            };
        let jwt = self.jwt_token().clone();
        let mail_box_clone = mail_box.clone();
        existing_token.or_else(move |()|
            AccessToken::request(&jwt, installation_id)
                .inspect(move |access_token|
                    mail_box_clone.do_send(StoreToken {
                        installation_id,
                        access_token: access_token.clone(),
                    }))
                .map(|access_token| access_token.token)
        )
    }

    fn jwt_token(&mut self) -> Arc<String> {
        match self.jwt {
            Some(ref jwt) if jwt.is_close_to_expiry() => jwt.token.clone(),
            Some(_) | None => {
                let new_jwt = JwtState::generate(&self.private_key);
                let token = new_jwt.token.clone();
                self.jwt = Some(new_jwt);
                token
            }
        }
    }

    fn read_to_string(path: String) -> Result<Vec<u8>, error::Error> {
        let mut file = File::open(path)?;
        let mut content = vec![];
        file.read_to_end(&mut content)?;
        return Ok(content);
    }
}
