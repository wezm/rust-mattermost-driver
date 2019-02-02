use futures::{Future, Stream};
use hyper;
use hyper::{Body, Client as HyperClient, Request, Response, StatusCode, Uri};
use hyper_rustls::HttpsConnector;
use serde::{Deserialize, Serialize};
use serde_json;
use url::{self, Url};

use std::fmt;

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
