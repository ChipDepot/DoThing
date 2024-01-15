use anyhow::Result;
use reqwest::Request;
pub trait BuildRequest {
    fn build_request<I: AsRef<str>>(&self, domain: I) -> Result<Request>;
}
