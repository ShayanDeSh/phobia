use crate::Record;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct Event {
    host: String,
    path: String,
    body: crate::Body,
    start: u32,
    end: u32,
    step: usize,
    client: reqwest::Client,
}

impl Event {
    async fn run(&self) -> Result<(), crate::Error> {
        match &self.body {
            crate::Body::MULTIPART { path } => {
                let response = self.send_file(path).await?;
            }
        }
//        for start in (self.start..self.end).step_by(self.step) {
//        }
        Ok(())
    }

    async fn send_file(&self, file: &str) 
        -> Result<reqwest::Response, crate::Error> {
        let mut file = File::open(file).await?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        let part = reqwest::multipart::Part::bytes(contents);
        let form = reqwest::multipart::Form::new()
            .part("file", part);
        let response = self.client.post(format!("{}{}", self.host, self.path))
            .multipart(form)
            .send().await?;
        Ok(response)
    }

    pub fn new(record: Record, scale: u32, step: usize) -> Event {
        Event {
            host: record.host,
            path: record.path,
            body: record.body,
            start: record.start / scale,
            end: record.end / scale,
            step: step,
            client: reqwest::Client::new(),
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
            end: 2223000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART { path: "./tests/data/test_data.json".into() }
        };

        let event = Event::new(record, 20, 1000);
        let response = event.send_file("./tests/data/test_data.json".into())
                            .await?;
        println!("status: {:?}", response.status());
        mock.assert();
        assert!(response.status() == reqwest::StatusCode::OK);
        Ok(())
    }
}
