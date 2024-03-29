/*
This functionality (but not code, necessarily) is duplicated in status_check.rs

The two should be merged, but it may be necessary to multithread this first (as this promises to be incredibly time intensive.)
*/

use futures::executor::block_on;
use reqwest;
use std::time::Duration;
// use http::{Response, Request, StatusCode };

// struct StatusCode {
//     url:
// }

pub async fn async_check_status_code(url: String) -> Result<reqwest::StatusCode, reqwest::Error> {
    let timeout = Duration::new(5, 0);
    let client = reqwest::blocking::Client::new();
    let res = client.head(&url).send()?;
    Ok(res.status())
}

pub fn check_status_code(url: String) -> Result<reqwest::StatusCode, reqwest::Error> {
    let timeout = Duration::new(5, 0);
    let client = reqwest::blocking::Client::new();
    let res = client.head(&url).send()?;
    Ok(res.status())
}

#[test]
fn test_check_status_code() {
    assert!(true)
}
