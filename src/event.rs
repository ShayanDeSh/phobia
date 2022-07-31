use crate::Record;
use reqwest::Method;
use std::borrow::Cow;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::task::JoinHandle;
use tracing::info;

#[derive(Debug)]
pub struct Event {
    pub record: Record,
    step: usize,
    scale: u32,
    joins: Vec<JoinHandle<()>>,
}

impl Event {
    pub async fn run(&mut self) -> Result<(), crate::Error> {
        match &self.record.body {
            crate::Body::MULTIPART { path, name } => {
                let filename = path.rsplit_once("/").unwrap_or(("", path)).1;
                let mut file = File::open(path).await?;
                let mut contents = vec![];
                file.read_to_end(&mut contents).await?;
                let mime_type = infer::get(&contents).map_or("text/plain", |mime| mime.mime_type());
                for step in (self.record.start..self.record.end).step_by(self.step) {
                    let c = contents.clone();
                    let r = self.record.clone();
                    let n = name.clone();
                    let m = mime_type.to_string();
                    let f = filename.to_string();
                    info!("Step {step} of {:?}", r);
                    let join = tokio::spawn(async move {
                        let response = Self::send_file(
                            r.method.clone(),
                            format!("{:}{:}", r.host, r.path),
                            n,
                            m.into(),
                            f.into(),
                            c.into(),
                        )
                        .await;
                        match response {
                            Ok(resp) => info!("Status of {:?}: {}", r, resp.status()),
                            Err(err) => info!("Error with step {step} of {:?}: {err}", r)
                        }
                    });
                    tokio::time::sleep(tokio::time::Duration::from_secs(self.step as u64)).await;
                    self.joins.push(join);
                }
            }
        }
        println!("returning");
        Ok(())
    }

    async fn send_file(
        method: String,
        url: String,
        name: String,
        mime_type: String, 
        filename: String,
        buf: Cow<'static, [u8]>,
    ) -> Result<reqwest::Response, crate::Error> {
        let client = reqwest::Client::new();
        let part = reqwest::multipart::Part::bytes(buf).file_name(filename).mime_str(&mime_type)?;
        let form = reqwest::multipart::Form::new().part(name, part);
        let response = client
            .request(Method::from_bytes(method.as_bytes())?, url)
            .multipart(form)
            .send()
            .await?;
        Ok(response)
    }

    pub async fn wait(self) -> Result<(), crate::Error> {
        for join in self.joins.into_iter() {
            tokio::join!(join).0?;
        }
        Ok(())
    }

    pub fn new(mut record: Record, scale: u32, step: usize) -> Event {
        record.start /= scale;
        record.end /= scale;
        Event {
            record,
            scale,
            step: step / scale as usize,
            joins: Vec::new(),
        }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.record.start == other.record.start {
            return self.record.end.cmp(&other.record.end);
        }
        self.record.start.cmp(&other.record.start)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.record.start.eq(&other.record.start) && self.record.end.eq(&other.record.end)
    }
}

impl Eq for Event {}

impl Clone for Event {
    fn clone(&self) -> Self {
        Event::new(self.record.clone(), self.scale, self.step);
        Event {
            record: self.record.clone(),
            scale: self.scale,
            step: self.step,
            joins: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event::Event;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_send_file() -> Result<(), crate::Error> {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/yolo/v2/predict")
                .body_contains("file");
            then.status(200);
        });

        let record = crate::Record {
            method: "POST".into(),
            host: server.base_url(),
            start: 0,
            end: 8,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path:"./tests/data/test_data.yaml".into(),
                name: "file".into(),
            },
        };

        let mut event = Event::new(record, 2, 2);
        event.run().await?;
        for join in event.joins.into_iter() {
            let _ = tokio::join!(join);
        }
        mock.assert_hits(4);
        Ok(())
    }
}
