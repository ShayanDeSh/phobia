use serde::Deserialize;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub mod generator;
pub mod event;

#[derive(Deserialize, Debug)]
pub struct Record {
    method: String,
    host: String,
    path: String,
    start: u32,
    end: u32,
    #[serde(flatten)]
    body: Body,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "content-type", content = "body")]
pub enum Body {
    #[cfg(__YES__)]
    #[serde(rename(deserialize = "json"))]
    JSON,
    #[cfg(__YES__)]
    #[serde(rename(deserialize = "form"))]
    FORM,
    #[serde(rename(deserialize = "multipart"))]
    MULTIPART { path: String },
}
