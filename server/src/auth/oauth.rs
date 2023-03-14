use chrono::{DateTime, Utc};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, url::Url, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, RequestTokenError, Scope, StandardTokenResponse,
    TokenResponse, TokenUrl,
};

static ENV_PREFIX: &str = "AUTH_";
pub fn env(name: &str) -> String {
    let mut prefixed = ENV_PREFIX.to_string();
    prefixed.push_str(name);
    std::env::var(&prefixed)
        .unwrap_or_else(|_| panic!("Missing environment variable '{}'", prefixed))
}

pub struct Provider {
    client: BasicClient,
    scope: String,
    introspector: Introspector,
}
impl Provider {
    //"https://discord.com/api/oauth2/authorize?response_type=code
    //"https://discord.com/api/oauth2/token",

    pub fn github() -> Self {
        Self {
            client: BasicClient::new(
                ClientId::new(env(concat!("GITHUB", "_CLIENT_ID"))),
                Some(ClientSecret::new(env(concat!("GITHUB", "_CLIENT_SECRET")))),
                AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
                Some(
                    TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                        .unwrap(),
                ),
            )
            .set_redirect_uri(Self::redirect_url("github")),
            scope: "user:email".to_string(),
            introspector: Introspector::Github,
        }
    }
    fn redirect_url(name: &str) -> RedirectUrl {
        let mut base = ENV_PREFIX.to_string();
        base.push_str("BASE_URL");
        base = std::env::var(base).unwrap_or_else(|_| "http://127.0.0.1:3000/auth".into());
        base.push_str("/callback/");
        base.push_str(name);
        RedirectUrl::new(base).unwrap()
    }

    pub fn authorize_url(&self) -> (Url, CsrfToken) {
        self.client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(self.scope.to_string()))
            .url()
    }
    pub async fn exchange_code(&self, code: String) -> Result<UserData, Error> {
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|err| match err {
                RequestTokenError::ServerResponse(_) => Error::Unauthorized,
                _ => Error::internal(err),
            })?;
        self.introspector.call(token).await
    }
}

#[derive(Debug, Copy, Clone)]
enum Introspector {
    Github,
}
impl Introspector {
    #[tracing::instrument(name = "introspect", skip(token))]
    async fn call(
        self,
        token: StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    ) -> Result<UserData, Error> {
        match self {
            Introspector::Github => {
                let client = reqwest::Client::builder()
                    .user_agent(env!("CARGO_PKG_NAME"))
                    .build()
                    .unwrap();
                let user_request = client
                    .get("https://api.github.com/user")
                    .bearer_auth(token.access_token().secret())
                    .send();
                let email_request = client
                    .get("https://api.github.com/user/emails")
                    .bearer_auth(token.access_token().secret())
                    .send();

                let user = user_request
                    .await
                    .map_err(Error::from)?
                    .json::<github::User>()
                    .await
                    .map_err(Error::internal)?;
                let emails = email_request
                    .await
                    .map_err(Error::from)?
                    .json::<Vec<github::Email>>()
                    .await
                    .map_err(Error::internal)?;

                Ok(UserData {
                    provider: "github".to_string(),
                    id: user.id.to_string(),
                    login: user.login.clone(),
                    name: user.name.unwrap_or_else(|| user.login.clone()),
                    email: emails
                        .iter()
                        .find(|email| email.verified && email.primary)
                        .or_else(|| emails.iter().find(|email| email.verified))
                        .ok_or(Error::Unauthorized)?
                        .email
                        .clone(),
                    expires: Utc::now() + chrono::Duration::weeks(3),
                })
            }
        }
    }
}

pub struct UserData {
    pub provider: String,
    /// Unique stable identifier
    pub id: String,
    /// Unique username
    pub login: String,
    /// Full name
    pub name: String,
    /// Primary email
    pub email: String,
    pub expires: DateTime<Utc>,
}

pub enum Error {
    Internal,
    Unauthorized,
}
impl Error {
    fn internal<E: std::error::Error>(err: E) -> Self {
        tracing::warn!(%err);
        Self::Internal
    }
}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.status().map_or(false, |code| code.is_client_error()) {
            Self::Unauthorized
        } else {
            Self::internal(err)
        }
    }
}

mod github {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct User {
        pub login: String,
        pub id: i64,
        pub name: Option<String>,
        //avatar_url: String,
        //...
    }
    #[derive(Deserialize)]
    pub struct Email {
        pub email: String,
        pub primary: bool,
        pub verified: bool,
        //visibility
    }
}
