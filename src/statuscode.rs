use reqwest;
use std::time::Duration;
use reqwest::ClientBuilder;
// use http::{Response, Request, StatusCode };
    
// struct StatusCode {
//     url: 
// }

pub async fn check_status_code(url: &str) -> Result<reqwest::StatusCode, reqwest::Error > {
    let timeout = Duration::new(5, 0);
    let client = ClientBuilder::new().timeout(timeout).build()?;
    let res = client.head(url).send().await?;
    Ok(res.status())
}
