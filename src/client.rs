use futures::{Future, Stream};
use hyper;
use hyper::{Body, Client as HyperClient, Request, Response, StatusCode, Uri};
use hyper_rustls::HttpsConnector;
use serde::{Deserialize, Serialize};
use serde_json;
use url::{self, Url};

use std::fmt;

use crate::channel;
use crate::team;
use crate::user;

const DNS_WORKER_THREADS: usize = 4;
const TOKEN: &'static str = "token";

#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    Http(hyper::http::Error),
    Json(serde_json::Error),
    Url(url::ParseError),
    InvalidUrl,
    Response(ErrorBody),
    Fixme,
    InvalidStr,
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::Url(error)
    }
}

impl From<hyper::http::Error> for Error {
    fn from(error: hyper::http::Error) -> Self {
        Error::Http(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Error::Hyper(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Json(error)
    }
}

#[derive(Serialize)]
struct Login {
    login_id: String,
    password: String,
    token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorBody {
    pub id: String,
    pub message: String,
    pub request_id: String,
    pub status_code: i64,
    pub is_oath: bool,
}

pub struct UnauthenticatedClient {
    http: HttpClient,
}

struct SessionToken(String);

struct HttpClient {
    base_url: Url,
    hyper: HyperClient<HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
}

pub struct Client {
    http: HttpClient,
    session_token: SessionToken,
}

impl SessionToken {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MattermostClient")
    }
}

impl HttpClient {
    fn post<B>(&self, path: &str, body: B) -> impl Future<Item = Response<Body>, Error = Error>
    where
        B: Into<Body>,
    {
        let request_url = self.base_url.join(path);

        futures::future::ok(self.hyper.clone())
            .and_then(|client| request_url.map(|url| (url, client)).map_err(Error::from))
            .and_then(|(url, client)| {
                eprintln!("POST {}", url.as_str());
                let mut request = Request::post(url.as_str());

                client
                    .request(request.body(body.into()).expect("FIXME"))
                    .map_err(Error::from)
            })
    }

    fn get(
        &self,
        path: &str,
        session_token: &SessionToken,
    ) -> impl Future<Item = Response<Body>, Error = Error> {
        let request_url = self.base_url.join(path);
        let authorization = format!("Bearer {}", session_token.as_str());

        futures::future::ok(self.hyper.clone())
            .and_then(|client| request_url.map(|url| (url, client)).map_err(Error::from))
            .and_then(|(url, client)| {
                eprintln!("GET {}", url.as_str());
                let mut request = Request::get(url.as_str());
                request.header(hyper::header::AUTHORIZATION, authorization);

                client
                    .request(request.body(Body::empty()).expect("FIXME"))
                    .map_err(Error::from)
            })
    }
}

impl UnauthenticatedClient {
    pub fn new(url: Url) -> Result<Self, Error> {
        if url.scheme() != "https" {
            return Err(Error::InvalidUrl);
        }

        // Append the api base
        let url = url.join("/api/v4/")?;

        let https = HttpsConnector::new(DNS_WORKER_THREADS);
        let client: HyperClient<_, hyper::Body> = HyperClient::builder().build(https);

        Ok(UnauthenticatedClient {
            http: HttpClient {
                base_url: url,
                hyper: client,
            },
        })
    }

    /// Consume an UnauthenticatedClient and return a Client if successful
    pub fn authenticate(
        self,
        login_id: String,
        password: String,
        token: Option<String>,
    ) -> impl Future<Item = Client, Error = Error> {
        // Construct body
        let body = Login {
            login_id,
            password,
            token,
        };

        // Send request
        self.http
            .post("users/login", serde_json::to_string(&body).unwrap())
            .inspect(|res| {
                eprintln!("Status:\n{}", res.status());
                eprintln!("Headers:\n{:#?}", res.headers());
            })
            .and_then(|res| {
                res.headers()
                    .get(TOKEN)
                    .ok_or_else(|| Error::Fixme)
                    .and_then(|token| {
                        token
                            .to_str()
                            .map(|token| token.to_string())
                            .map_err(|_err| Error::InvalidStr)
                    })
                    .map(|token| (res, token))
            })
            .and_then(|(res, token)| {
                res.into_body()
                    .concat2()
                    .map_err(Error::from)
                    .map(|body| (body, token))
            })
            .and_then(|(body, token)| {
                let b = std::str::from_utf8(&body).unwrap();
                eprintln!("body = {}", b);
                let user = serde_json::from_slice::<user::User>(&body); //.or_else(|| serde_json::from_slice::<ErrorBody>(&body))
                dbg!(&token);

                futures::future::ok(Client {
                    http: self.http,
                    session_token: SessionToken(token),
                })
            })
    }
}

impl Client {
    pub fn get_user_teams(
        &self,
        user_id: user::UserParam,
    ) -> impl Future<Item = Vec<team::Team>, Error = Error> {
        self.http
            .get(
                &format!("users/{}/teams", user_id.as_str()),
                &self.session_token,
            )
            .inspect(|res| {
                eprintln!("Status:\n{}", res.status());
                eprintln!("Headers:\n{:#?}", res.headers());
            })
            .and_then(|res| res.into_body().concat2().map_err(Error::from))
            .and_then(|body| {
                let b = std::str::from_utf8(&body).unwrap();
                eprintln!("body = {}", b);
                serde_json::from_slice::<Vec<team::Team>>(&body).map_err(Error::from)
            })
    }

    pub fn get_team_channels_for_user(
        &self,
        team_id: &team::TeamId,
        user_id: user::UserParam,
    ) -> impl Future<Item = Vec<channel::Channel>, Error = Error> {
        self.http
            .get(
                &format!(
                    "users/{}/teams/{}/channels",
                    user_id.as_str(),
                    team_id.as_str()
                ),
                &self.session_token,
            )
            .inspect(|res| {
                eprintln!("Status:\n{}", res.status());
                eprintln!("Headers:\n{:#?}", res.headers());
            })
            .and_then(|res| res.into_body().concat2().map_err(Error::from))
            .and_then(|body| {
                let b = std::str::from_utf8(&body).unwrap();
                eprintln!("body = {}", b);
                serde_json::from_slice::<Vec<channel::Channel>>(&body).map_err(Error::from)
            })
    }
}
