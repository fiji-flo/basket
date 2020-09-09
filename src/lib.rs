//! Rust client library for [basket](https://github.com/mozmeao/basket/)
//! Documentation can be found at [http://basket.readthedocs.org/].
use failure::Error;
use failure::Fail;
use reqwest::Client;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use std::fmt;
use url::Url;

#[derive(Fail, Debug)]
pub enum BasketError {
    #[fail(display = "token must be a uuid")]
    InvalidTokenFormat,
}

#[serde(rename_all = "lowercase")]
#[derive(Deserialize, PartialEq, Debug)]
pub enum Status {
    Ok,
    Error,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Ok => write!(f, "ok"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[serde(rename_all = "lowercase")]
#[derive(Deserialize, Debug, Fail)]
pub struct ApiResponse {
    status: Status,
    #[serde(flatten)]
    data: Value,
}

impl fmt::Display for ApiResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Status::Ok if !self.data.is_null() => {
                write!(f, "{}", serde_json::to_string(&self.data).unwrap())
            }
            _ => write!(f, "{}", self.status),
        }
    }
}

#[derive(Serialize)]
pub enum Format {
    H,
    T,
}

impl Default for Format {
    fn default() -> Self {
        Self::H
    }
}

#[derive(Serialize)]
pub struct Subscribe {
    email: String,
    newsletters: String,
    #[serde(flatten)]
    opts: Option<SubscribeOpts>,
}

#[derive(Serialize)]
pub struct Unsubscribe {
    email: String,
    newsletters: String,
    optout: bool,
}

#[derive(Serialize, Default)]
pub struct SubscribeOpts {
    format: Option<Format>,
    country: Option<String>,
    lang: Option<String>,
    optin: Option<bool>,
    source_url: Option<String>,
    trigger_welcome: Option<bool>,
    sync: Option<bool>,
}

#[derive(Serialize)]
pub struct UpdateUser {
    email: Option<String>,
    #[serde(flatten)]
    opts: Option<UpdateUserOpts>,
}

#[derive(Serialize, Default)]
pub struct UpdateUserOpts {
    format: Option<Format>,
    country: Option<String>,
    lang: Option<String>,
    optin: Option<bool>,
    newsletters: Option<String>,
}

#[derive(Serialize)]
struct DebugUser {
    email: String,
    supertoken: String,
}

#[derive(Serialize)]
struct LookupUser {
    email: String,
    #[serde(rename = "api-key")]
    api_key: String,
}

#[derive(Serialize)]
struct Recover {
    email: String,
}

#[derive(Clone)]
pub struct Basket {
    api_key: String,
    basket_url: Url,
    client: Client,
}

impl Basket {
    pub fn new(api_key: impl Into<String>, basket_url: Url) -> Self {
        Basket {
            api_key: api_key.into(),
            basket_url,
            client: Client::new(),
        }
    }
}

impl Basket {
    pub async fn subscribe(
        &self,
        email: impl Into<String>,
        newsletters: Vec<String>,
        opts: Option<SubscribeOpts>,
    ) -> Result<(), Error> {
        let form = Subscribe {
            email: email.into(),
            newsletters: newsletters.join(","),
            opts,
        };

        let res = self
            .client
            .post(self.basket_url.join("/news/subscribe/")?)
            .form(&form)
            .send()
            .await?;

        match res.json::<ApiResponse>().await {
            Ok(r) if r.status == Status::Error => Ok(()),
            Ok(r) => Err(r.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn unsubscribe(
        &self,
        email: impl Into<String>,
        newsletters: Vec<String>,
        optout: bool,
    ) -> Result<(), Error> {
        let form = Unsubscribe {
            email: email.into(),
            newsletters: newsletters.join(","),
            optout,
        };

        let res = self
            .client
            .post(self.basket_url.join("/news/unsubscribe/")?)
            .form(&form)
            .send()
            .await?;

        match res.json::<ApiResponse>().await {
            Ok(r) if r.status == Status::Error => Ok(()),
            Ok(r) => Err(r.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_user(&self, token: impl AsRef<str>) -> Result<Value, Error> {
        let res = self
            .client
            .get(
                self.basket_url
                    .join(&format!("/news/user/{}/", token.as_ref()))?,
            )
            .send()
            .await?;
        match res.json::<ApiResponse>().await {
            Ok(r) if r.status == Status::Ok => Ok(r.data),
            Ok(r) => Err(r.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn update_user(
        &self,
        email: impl Into<String>,
        token: impl AsRef<str>,
        opts: Option<UpdateUserOpts>,
    ) -> Result<(), Error> {
        let form = UpdateUser {
            email: Some(email.into()),
            opts,
        };
        let res = self
            .client
            .post(
                self.basket_url
                    .join(&format!("/news/user/{}/", token.as_ref()))?,
            )
            .form(&form)
            .send()
            .await?;
        match res.json::<ApiResponse>().await {
            Ok(r) if r.status == Status::Ok => Ok(()),
            Ok(r) => Err(r.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn newsletters(&self) -> Result<Value, Error> {
        let res = self
            .client
            .get(self.basket_url.join("/news/newsletters/")?)
            .send()
            .await?;
        match res.json::<ApiResponse>().await {
            Ok(r) if r.status == Status::Ok => Ok(r.data),
            Ok(r) => Err(r.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn debug_user(
        &self,
        email: impl AsRef<str>,
        supertoken: impl AsRef<str>,
    ) -> Result<Value, Error> {
        let res = self
            .client
            .get(self.basket_url.join("/news/debug-user/")?)
            .query(&[
                ("email", email.as_ref()),
                ("supertoken", supertoken.as_ref()),
            ])
            .send()
            .await?;
        match res.json::<ApiResponse>().await {
            Ok(r) if r.status == Status::Ok => Ok(r.data),
            Ok(r) => Err(r.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn lookup_user(&self, email: impl AsRef<str>) -> Result<Value, Error> {
        let res = self
            .client
            .get(self.basket_url.join("/news/lookup-user/")?)
            .query(&[
                ("email", email.as_ref()),
                ("api-key", self.api_key.as_str()),
            ])
            .send()
            .await?;
        match res.json::<ApiResponse>().await {
            Ok(r) if r.status == Status::Ok => Ok(r.data),
            Ok(r) => Err(r.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn recover(&self, email: impl Into<String>) -> Result<(), Error> {
        let form = Recover {
            email: email.into(),
        };
        let res = self
            .client
            .post(self.basket_url.join("/news/recover/")?)
            .form(&form)
            .send()
            .await?;
        match res.json::<ApiResponse>().await {
            Ok(r) if r.status == Status::Ok => Ok(()),
            Ok(r) => Err(r.into()),
            Err(e) => Err(e.into()),
        }
    }
}
