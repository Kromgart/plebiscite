use const_format::concatcp;
//use wasm_bindgen::{prelude::JsCast, JsValue};
//use wasm_bindgen_futures::JsFuture;

//--------------------------------------------------------------------

pub fn log_str(s: &str) {
    web_sys::console::log_1(&s.into()) 
}

#[macro_export]
macro_rules! log_pfx {
    ($pfx:literal, $str:literal) => { 
        log_str(concatcp!($pfx, ": ", $str));
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

type StdError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub enum FetchError<SE, DE> {
    NoWindow,
    FetchFailed,
    ResponseNotOk,
    ReadBodyFailed,
    Deserialize(DE),
    Serialize(SE),
    Other(String),
}

pub type FetchResult<T, SE, DE> = Result<T, FetchError<SE, DE>>;

//--------------------------------------------------------------------

#[derive(Copy, Clone)]
enum Verb {
    Get,
    Post
}

impl Verb {
    fn to_str(self) -> &'static str {
        match self {
            Verb::Get => "GET",
            Verb::Post => "POST",
        }
    }
}

//--------------------------------------------------------------------
/*
pub async fn get_json<T>(url: &str) -> FetchResult<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    fetch_json(url, Verb::Get, <Option<&()>>::None).await
}

pub async fn post_json<T, B>(url: &str, body: &B) -> FetchResult<T>
where
    T: for<'a> serde::Deserialize<'a>,
    B: serde::Serialize,
{
    fetch_json(url, Verb::Post, Some(body)).await
}
*/
//--------------------------------------------------------------------
/*
async fn fetch_json<T, B>(url: &str, verb: Verb, body: Option<&B>) -> FetchResult<T>
where
    T: for<'a> serde::Deserialize<'a>,
    B: serde::Serialize,
{
    fetch(
        url, 
        verb, 
        body.map(|b| (b, "application/json")), 
        |buf| {
            serde_json::from_slice(buf)
                .map_err(|e| Box::new(e) as StdError)
        }
    )
    .await
}
*/
//--------------------------------------------------------------------

async fn fetch<F, T, B>(url: &str, verb: Verb, body: Option<(&B, &'static str)>, mk_result: F) -> FetchResult<T, StdError, StdError>
where
    F: FnOnce(&[u8]) -> Result<T, StdError>,
    B: serde::Serialize,
{
    log!("fetch {} / {}", url, verb.to_str());

    use FetchError as ER;

    let wnd = web_sys::window().ok_or(ER::NoWindow)?;

    let mut opts = web_sys::RequestInit::new();

    opts.method(verb.to_str());

    if let Some((body, content_type)) = body {
        //let body = serde_wasm_bindgen::to_value(body).map_err(ER::Serialize)?;
        let body = serde_json::to_string(body).map_err(|e| ER::Serialize(Box::new(e)))?;
        //web_sys::console::debug_1(&body);
        opts.body(Some(&JsValue::from_str(&body)));

        let headers = web_sys::Headers::new().expect("Cannot create fetch Headers");
        headers.set("Content-Type", content_type).expect("Cannot set fetch Content-Type header");
        opts.headers(&headers);
    }

    opts.credentials(web_sys::RequestCredentials::SameOrigin);
    opts.mode(web_sys::RequestMode::SameOrigin);


    let resp = JsFuture::from(wnd.fetch_with_str_and_init(&url, &opts))
        .await
        .map_err(|_| ER::FetchFailed)?;
    let resp: web_sys::Response = resp.dyn_into().expect("fetch() didn't produce a Response");

    if !resp.ok() {
        log!("fetch NOT OK");
        Err(ER::ResponseNotOk)
    } else {
        log!("fetch OK, processing response...");

        let arr_buf = JsFuture::from(
            resp.array_buffer()
                .expect("Response.arrayBuffer() didn't produce a Promise"),
        )
        .await
        .map_err(|_| ER::ReadBodyFailed)?;

        let buf = js_sys::Uint8Array::new(&arr_buf).to_vec();
        let result = mk_result(buf.as_slice()).map_err(ER::Deserialize);

        log!("fetch deserialized and finished");
        result
    }
}
