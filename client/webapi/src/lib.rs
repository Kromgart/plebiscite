use wasm_bindgen::{prelude::JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use plebiscite_types::{Usergroup, UsergroupId, UsergroupData};

type StdError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub enum FetchError {
    NoWindow,
    FetchFailed,
    ResponseNotOk,
    ReadBodyFailed,
    Deserialize(StdError),
    Serialize(StdError),
    Other(String),
}

pub type FetchResult<T> = Result<T, FetchError>;

pub async fn get_assigned_usergroups() -> FetchResult<Vec<Usergroup>> {
    get_json("/api/user/groups").await
}

pub async fn create_usergroup(data: &UsergroupData) -> FetchResult<UsergroupId> {
    post_json("/api/user/groups/create", data).await
}

//--------------------------------------------------------------------

enum Verb {
    Get,
    Post
}

async fn get_json<T>(url: &str) -> FetchResult<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    fetch_json(url, Verb::Get, <Option<&()>>::None).await
}

async fn post_json<T, B>(url: &str, body: &B) -> FetchResult<T>
where
    T: for<'a> serde::Deserialize<'a>,
    B: serde::Serialize,
{
    fetch_json(url, Verb::Post, Some(body)).await
}

//--------------------------------------------------------------------

async fn fetch_json<T, B>(url: &str, verb: Verb, body: Option<&B>) -> FetchResult<T>
where
    T: for<'a> serde::Deserialize<'a>,
    B: serde::Serialize,
{
    fetch(url, verb, body, |buf| {
        serde_json::from_slice(buf).map_err(|e| Box::new(e) as StdError)
    })
    .await
}

//--------------------------------------------------------------------

async fn fetch<F, T, B>(url: &str, verb: Verb, body: Option<&B>, mk_result: F) -> FetchResult<T>
where
    F: FnOnce(&[u8]) -> Result<T, StdError>,
    B: serde::Serialize,
{
    use FetchError as ER;

    let wnd = web_sys::window().ok_or(ER::NoWindow)?;

    let mut opts = web_sys::RequestInit::new();

    opts.method(match verb {
        Verb::Get => "GET",
        Verb::Post => "POST",
    });

    if let Some(body) = body {
        //let body = serde_wasm_bindgen::to_value(body).map_err(ER::Serialize)?;
        let body = serde_json::to_string(body).map_err(|e| ER::Serialize(Box::new(e)))?;
        //web_sys::console::debug_1(&body);
        opts.body(Some(&JsValue::from_str(&body)));

        let headers = web_sys::Headers::new().expect("Cannot create fetch Headers");
        headers.set("Content-Type", "application/json").expect("Cannot set fetch Content-Type header");
        opts.headers(&headers);
    }

    opts.credentials(web_sys::RequestCredentials::SameOrigin);
    opts.mode(web_sys::RequestMode::SameOrigin);

    let resp = JsFuture::from(wnd.fetch_with_str_and_init(&url, &opts))
        .await
        .map_err(|_| ER::FetchFailed)?;
    let resp: web_sys::Response = resp.dyn_into().expect("fetch() didn't produce a Response");

    if !resp.ok() {
        Err(ER::ResponseNotOk)
    } else {
        //let content_len = resp.headers().get("Content-Length").unwrap().expect("Response is missing Content-Length");
        //let content_len = <usize as std::str::FromStr>::from_str(content_len.as_str()).expect("Could not parse Content-Length as usize");

        let arr_buf = JsFuture::from(
            resp.array_buffer()
                .expect("Response.arrayBuffer() didn't produce a Promise"),
        )
        .await
        .map_err(|_| ER::ReadBodyFailed)?;

        let buf = js_sys::Uint8Array::new(&arr_buf).to_vec();
        let result = mk_result(buf.as_slice()).map_err(ER::Deserialize);

        result
    }
}
