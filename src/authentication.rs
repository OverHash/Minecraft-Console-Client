use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cache::Cache;

/// The Azure Application client ID
const CLIENT_ID: &str = "54473e32-df8f-42e9-a649-9419b0dab9d3";

/// The response from authenticating with Microsoft OAuth flow
#[derive(Deserialize, Serialize)]
struct MicrosoftTokenAuthorizeResponse {
    /// The type of token for authentication
    token_type: String,
    /// The scope we have access to
    scope: String,
    /// Seconds until the authentication token expires
    expires_in: u32,
    /// Seconds until the authentication token expires
    ext_expires_in: u32,
    /// The authentication token itself
    access_token: String,
    /// The token used for refreshing access
    refresh_token: String,
    /// The ID of the token
    id_token: String,
}

/// The response from Xbox when authenticating with a Microsoft token
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct XboxLiveAuthenticationResponse {
    /// An ISO-8601 timestamp of when the token was issued
    issue_instant: String,
    /// An ISO-8601 timestamp of when the token expires
    not_after: String,
    /// The xbox authentication token to use
    token: String,
    /// An object that contains a vec of `uhs` objects
    /// Looks like { "xui": [{"uhs": "xbl_token"}] }
    display_claims: HashMap<String, Vec<HashMap<String, String>>>,
}

/// The response from Minecraft when attempting to authenticate with an xbox token
#[derive(Deserialize, Serialize, Debug)]
struct MinecraftAuthenticationResponse {
    /// Some UUID of the account
    username: String,
    /// The minecraft JWT access token
    access_token: String,
    /// The type of access token
    token_type: String,
    /// How many seconds until the token expires
    expires_in: u32,
}

/// The response from Minecraft when attempting to retrieve a users profile
#[derive(Serialize, Deserialize, Debug)]
struct MinecraftProfileResponse {
    /// The UUID of the account
    id: String,
    /// The name of the user
    name: String,
}

pub struct TokenResult {
    pub minecraft_token: String,
    pub retrieve_type: RetrieveType,
}

pub enum RetrieveType {
    FromCache,
    FromUserLogin {
        microsoft_refresh_token: String,
        expires_in: u32,
    },
}

async fn microsoft_authenticate_token<T>(
    client: &Client,
    data: T,
) -> Result<MicrosoftTokenAuthorizeResponse, Box<dyn std::error::Error>>
where
    T: Serialize + Sized,
{
    let authorization_token = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&data)
        .send()
        .await?
        .json()
        .await?;

    Ok(authorization_token)
}

fn get_auth_code<R>(mut reader: R) -> Result<String, Box<dyn std::error::Error>>
where
    R: BufRead,
{
    print!("Authorization code: ");
    io::stdout().flush()?;

    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;

    Ok(buffer)
}

/// Attempts to authenticate with Mojang and Minecraft servers, using the current cache if it exists.
/// Returns the Minecraft token.
pub async fn authenticate<R>(
    client: &Client,
    reader: R,
    cache: Option<&Cache>,
) -> Result<TokenResult, Box<dyn std::error::Error>>
where
    R: BufRead,
{
    // if the cache exists, let's check to see if the minecraft token has expired or not
    if let Some(cache) = cache {
        let cached_token = cache.get_minecraft_token();

        if let Some(token) = cached_token {
            println!("Cached token was valid!");
            return Ok(TokenResult {
                minecraft_token: token,
                retrieve_type: RetrieveType::FromCache,
            });
        }

        println!("Cached token was invalid, generating a new token...");
    }

    // step 1: get authorization token
    // if the cache exists, we can use the microsoft `refresh_token` to skip user authorization again
    let authorization_token = if let Some(cache) = cache {
        microsoft_authenticate_token(
            client,
            vec![
                ("client_id", CLIENT_ID),
                ("refresh_token", cache.get_microsoft_refresh_token()),
                ("grant_type", "refresh_token"),
                ("redirect_uri", "https://mccteam.github.io/redirect.html"),
            ],
        )
        .await?
    } else {
        // attempt to login to microsoft account (OAuth flow)
        // requires authorization from the user
        println!("Please login with your Microsoft account in the following link and retrieve the authorization code: https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?client_id={client_id}&response_type=code&scope=XboxLive.signin%20offline_access", client_id=CLIENT_ID);

        // retrieve the code from them the user
        let code = get_auth_code(reader)?;

        microsoft_authenticate_token(
            client,
            vec![
                ("client_id", CLIENT_ID),
                ("code", &code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", "https://mccteam.github.io/redirect.html"),
            ],
        )
        .await?
    };

    // step 3: authenticate with xbox live
    let xbox_authenticate_json = json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": &format!("d={}", authorization_token.access_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let xbox_resp: XboxLiveAuthenticationResponse = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&xbox_authenticate_json)
        .send()
        .await?
        .json()
        .await?;

    let xbox_token = &xbox_resp.token;
    let user_hash = &xbox_resp.display_claims["xui"][0]["uhs"];

    // step 4: convert xbox token into xbox security token
    let xbox_security_token_resp: XboxLiveAuthenticationResponse = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbox_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?
        .json()
        .await?;

    // step 5: authenticate with minecraft
    let minecraft_resp: MinecraftAuthenticationResponse = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&json!({
            "identityToken":
                format!(
                    "XBL3.0 x={user_hash};{xsts_token}",
                    user_hash = user_hash,
                    xsts_token = xbox_security_token_resp.token
                )
        }))
        .send()
        .await?
        .json()
        .await?;

    Ok(TokenResult {
        minecraft_token: minecraft_resp.access_token,
        retrieve_type: RetrieveType::FromUserLogin {
            microsoft_refresh_token: authorization_token.refresh_token,
            expires_in: authorization_token.expires_in,
        },
    })
}
