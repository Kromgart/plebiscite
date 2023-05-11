use plebiscite_types::{ Usergroup };
use wasm_bindgen::prelude::JsCast;
use wasm_bindgen_futures::JsFuture;


/*
#[wasm_bindgen()]
extern "C" {
    fn update_vector(xs: &mut [u8]);
    async fn read_into_slice(stream: &js_sys::Object, buf: &mut [u8]);
}

fn test_mut_slice() {
    let mut test = vec![1_u8; 10];
    update_vector(&mut test);

    unsafe {
        let test2 = js_sys::Uint8Array::view(test.as_slice());
        web_sys::console::log_1(&test2);
    }
}
*/

//use serde::de::Error;

type StdError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub enum FetchError {
    NoWindow,
    FetchFailed,
    ResponseNotOk,
    ReadBodyFailed,
    Deserialize(StdError),
    Other(String),
}


async fn fetch_data<F, T>(url: &str, deser: F) -> Result<T, FetchError> 
where F: FnOnce(&[u8]) -> Result<T, StdError>,
      T: for<'a> serde::de::Deserialize<'a>
{
    use FetchError as ER;

    let wnd = web_sys::window().ok_or(ER::NoWindow)?;

    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    opts.credentials(web_sys::RequestCredentials::SameOrigin);
    opts.mode(web_sys::RequestMode::SameOrigin);

    let resp = JsFuture::from(wnd.fetch_with_str_and_init(&url, &opts)).await
        .map_err(|_| ER::FetchFailed)?;
    let resp: web_sys::Response = resp.dyn_into().expect("fetch() didn't produce a Response");

    if !resp.ok() {
        Err(ER::ResponseNotOk)
    } else {
        //let content_len = resp.headers().get("Content-Length").unwrap().expect("Response is missing Content-Length");
        //let content_len = <usize as std::str::FromStr>::from_str(content_len.as_str()).expect("Could not parse Content-Length as usize");

        let arbu = JsFuture::from(resp.array_buffer().expect("Response.arrayBuffer() didn't produce a Promise"))
            .await
            .map_err(|_| ER::ReadBodyFailed)?;

        let buf = js_sys::Uint8Array::new(&arbu).to_vec();
        let result = deser(buf.as_slice()).map_err(ER::Deserialize);

        result
    }
}


async fn fetch_json<T>(url: &str) -> Result<T, FetchError>
where T: for<'a> serde::de::Deserialize<'a>
{
    fetch_data(
        url,
        |buf| serde_json::from_slice(buf).map_err(|e| Box::new(e) as StdError)
        ).await
}

pub async fn get_assigned_usergroups() -> Result<Vec<Usergroup>, FetchError> {
    fetch_json("/api/user/groups").await
}

