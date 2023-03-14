mod jwt;
mod oauth;
use axum::{
    extract::{FromRequest, Path, Query, TypedHeader},
    headers,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Extension, Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

pub fn nest(router: Router, path: &str) -> Router {
    let inner = Router::new()
        .route("/", get(list))
        .route("/html", get(list_html))
        .route("/logout", get(logout))
        .route("/login/:provider", get(login))
        .route("/callback/:provider", get(callback))
        .route("/whoami", get(whoami));
    router.nest(path, inner).layer(Extension(State::new_ref()))
}

struct State {
    providers: HashMap<String, oauth::Provider>,
    jwks: jwt::Keys,
}
type StateRef = std::sync::Arc<State>;
impl State {
    fn new() -> Self {
        Self {
            jwks: jwt::Keys::new(oauth::env("JWT_SECRET").as_bytes()),
            providers: HashMap::from([("github".to_string(), oauth::Provider::github())]),
        }
    }
    fn new_ref() -> StateRef {
        std::sync::Arc::new(Self::new())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Claims {
    iss: String,
    id: String,
    exp: i64,
}
impl fmt::Display for Claims {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.iss, self.id)
    }
}
pub use axum_extra::extract::Cached as Get;
pub type User = Get<Claims>;

static COOKIE_NAME: &str = "SCALLION";
fn new_cookie<'c, V>(value: V) -> Cookie<'c>
where
    V: Into<std::borrow::Cow<'c, str>>,
{
    Cookie::build(COOKIE_NAME, value)
        .path("/")
        .http_only(true)
        .permanent()
        .finish()
}

pub struct AuthRejection;
impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrObj {
                err: "Wrong credentials",
            }),
        )
            .into_response()
    }
}
#[derive(Serialize)]
struct ErrObj {
    err: &'static str,
}

#[axum::async_trait]
impl<B: Send> FromRequest<B> for Claims {
    type Rejection = AuthRejection;

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request(req).await.unwrap();
        let beader =
            TypedHeader::<headers::Authorization<headers::authorization::Bearer>>::from_request(
                req,
            )
            .await;
        let jwt = beader.map(|b| b.token().to_owned()).or_else(|_| {
            jar.get(COOKIE_NAME)
                .map(|c| c.value().to_owned())
                .ok_or(Self::Rejection {})
        })?;
        let state = req.extensions().get::<StateRef>().unwrap();

        let claims = state
            .jwks
            .decode::<Claims>(&jwt)
            .or(Err(Self::Rejection {}))?;
        Ok(claims)
    }
}

async fn list(Extension(state): Extension<StateRef>) -> impl IntoResponse {
    Json::<Vec<_>>(state.providers.keys().map(|s| s.to_owned()).collect())
}
async fn list_html(Extension(state): Extension<StateRef>) -> impl IntoResponse {
    Html(state.providers.keys().fold(String::new(), |acc, s| {
        acc + "<a href=\"./login/" + s + "\">" + s + "</a>&nbsp"
    }))
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove(new_cookie("")), Redirect::to("/"))
}

async fn whoami(claims: Claims) -> impl IntoResponse {
    Json(claims.to_string())
}

async fn login(
    Path(name): Path<String>,
    Extension(state): Extension<StateRef>,
) -> impl IntoResponse {
    if let Some(provider) = state.as_ref().providers.get(&name) {
        let (auth_url, _csrf_token) = provider.authorize_url();
        //FIXME: store csrf
        Redirect::to(auth_url.as_ref()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[derive(Debug, Deserialize)]
struct CallbackArgs {
    code: String,
    state: String,
}
async fn callback(
    Path(name): Path<String>,
    Extension(state): Extension<StateRef>,
    Query(query): Query<CallbackArgs>,
    jar: CookieJar,
) -> Response {
    if let Some(provider) = state.as_ref().providers.get(&name) {
        _ = query.state;
        //FIXME: checkcsrf
        let user = match provider.exchange_code(query.code.clone()).await {
            Ok(u) => u,
            Err(err) => {
                return match err {
                    oauth::Error::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                    oauth::Error::Unauthorized => AuthRejection.into_response(),
                }
            }
        };

        let jwt = state
            .jwks
            .encode(&Claims {
                iss: user.provider,
                id: user.id,
                exp: user.expires.timestamp(),
            })
            .unwrap();

        (jar.add(new_cookie(jwt)), Redirect::to("/")).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
