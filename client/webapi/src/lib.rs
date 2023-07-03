use wasm_bindgen::{prelude::JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use serde::{Serialize, Deserialize};

use plebiscite_types::{Usergroup, UsergroupId, UsergroupData};

//--------------------------------------------------------------------

pub fn log_str(s: &str) {
    web_sys::console::log_1(&s.into()) 
}

#[macro_export]
macro_rules! log_pfx {
    ($pfx:literal, $str:literal) => { 
        log_str(&format!("{}: {}", $pfx, $str));
    };
    ($pfx:literal, $str:literal, $($args:tt)+) => {{
        let msg = format!("{}: {}", $pfx, format_args!($str, $($args)+));
        log_str(&msg);
    }};
}

macro_rules! log {
    ($($args:tt)+) => { 
        log_pfx!("webapi", $($args)+)
    };
}

//--------------------------------------------------------------------

pub trait TypeConverter {
    type Error;

    fn content_type() -> &'static str;
    fn serialize<T: Serialize>(x: T) -> Result<JsValue, Self::Error>;
    fn deserialize<T: for<'a> Deserialize<'a>>(buf: &[u8]) -> Result<T, Self::Error>;
}

//--------------------------------------------------------------------

#[derive(Debug)]
pub enum FetchError<TC: TypeConverter> {
    NoWindow,
    FetchFailed,
    ResponseNotOk,
    ReadBodyFailed,
    Deserialize(TC::Error),
    Serialize(TC::Error),
    Other(String),
}

pub type FetchResult<T, TC> = Result<T, FetchError<TC>>;

//--------------------------------------------------------------------

enum Method<T> {
    Get,
    Post(T),
}

impl<T> Method<T> {
    fn into_parts(self) -> (&'static str, Option<T>) {
        match self {
            Self::Get => ("GET", None),
            Self::Post(x) => ("POST", Some(x)),
        }
    }
}

//--------------------------------------------------------------------

pub struct WebAPI<TC> {
    _marker: std::marker::PhantomData<TC>
}

impl<TC: TypeConverter> WebAPI<TC> { 

    pub async fn get_assigned_usergroups() -> FetchResult<Vec<Usergroup>, TC> {
        Self::http_get("/api/user/groups").await
    }

    pub async fn create_usergroup(data: &UsergroupData) -> FetchResult<UsergroupId, TC> {
        Self::http_post("/api/user/groups/create", data).await
    }

    //---------------------------------------------------------------

    async fn http_get<T>(url: &str) -> FetchResult<T, TC>
    where T: for<'a> Deserialize<'a>
    {
        Self::fetch::<T, ()>(url, Method::Get).await
    }

    async fn http_post<T, U>(url: &str, body: U) -> FetchResult<T, TC>
    where U: Serialize,
          T: for<'a> Deserialize<'a>
    {
        Self::fetch(url, Method::Post(body)).await
    }

    //---------------------------------------------------------------

    async fn fetch<T, U>(url: &str, method: Method<U>) -> FetchResult<T, TC>
    where U: Serialize,
          T: for<'a> Deserialize<'a>
    {

        macro_rules! err {
            ($id:ident) => { <FetchError<TC>>::$id };
        }

        let (verb, body) = method.into_parts();

        log!("fetch {} / {}", url, verb);

        let wnd = web_sys::window().ok_or(err!(NoWindow))?;
        let mut opts = web_sys::RequestInit::new();
        opts.method(verb);

        if let Some(body) = body {
            let body = TC::serialize(body).map_err(err!(Serialize))?;
            opts.body(Some(&body));

            let headers = web_sys::Headers::new().expect("Cannot create fetch Headers");
            headers.set("Content-Type", TC::content_type()).expect("Cannot set fetch Content-Type header");
            opts.headers(&headers);
        }

        opts.credentials(web_sys::RequestCredentials::SameOrigin);
        opts.mode(web_sys::RequestMode::SameOrigin);

        let resp = JsFuture::from(wnd.fetch_with_str_and_init(url, &opts))
            .await
            .map_err(|_| err!(FetchFailed))?;
        let resp: web_sys::Response = resp.dyn_into().expect("fetch() didn't produce a Response");

        if !resp.ok() {
            log!("fetch NOT OK");
            Err(err!(ResponseNotOk))
        } else {
            log!("fetch OK, processing response...");

            let arr_buf = JsFuture::from(
                resp.array_buffer()
                    .expect("Response.arrayBuffer() didn't produce a Promise"),
            )
            .await
            .map_err(|_| err!(ReadBodyFailed))?;

            let buf = js_sys::Uint8Array::new(&arr_buf).to_vec();
            let result = TC::deserialize(buf.as_slice()).map_err(err!(Deserialize));

            log!("fetch deserialized and finished");
            result
        }
    }
}

//--------------------------------------------------------------------

#[derive(Debug)]
pub struct JsonTypeConverter {}

impl TypeConverter for JsonTypeConverter {
    type Error = serde_json::Error;

    fn content_type() -> &'static str {
        "application/json"
    }

    fn serialize<T: Serialize>(x: T) -> Result<JsValue, Self::Error> {
        serde_json::to_string(&x).map(|x| x.into())
    }

    fn deserialize<T: for<'a> Deserialize<'a>>(buf: &[u8]) -> Result<T, Self::Error> {
        serde_json::from_slice(buf)
    }
}

pub type JsonWebAPI = WebAPI<JsonTypeConverter>;



