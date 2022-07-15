use crate::Record;
use std::borrow::Cow;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::task::JoinHandle;

pub struct Event {
    record: Record,
    step: usize,
    client: reqwest::Client,
    joins: Vec<JoinHandle<()>>,
}

impl Event {
    async fn run(&mut self) -> Result<(), crate::Error> {
        match &self.record.body {
            crate::Body::MULTIPART { path } => {
                let mut file = File::open(path).await?;
                let mut contents = vec![];
                file.read_to_end(&mut contents).await?;
                for _ in (self.record.start..self.record.end).step_by(self.step) {
                    let c = contents.clone();
                    let r = self.record.clone();
                    let foo = tokio::spawn(async move {
                        let _response = Self::send_file(r, c.into()).await;
                    });
                    tokio::time::sleep(tokio::time::Duration::from_millis(self.step as u64)).await;
                    self.joins.push(foo);
                }
            }
        }
        Ok(())
    }

    async fn send_file(
        record: Record,
        buf: Cow<'static, [u8]>,
    ) -> Result<reqwest::Response, crate::Error> {
        let client = reqwest::Client::new();
        let part = reqwest::multipart::Part::bytes(buf);
        let form = reqwest::multipart::Form::new().part("file", part);
        let response = client
            .post(format!("{}{}", record.host, record.path))
            .multipart(form)
            .send()
            .await?;
        Ok(response)
    }

    pub fn new(mut record: Record, scale: u32, step: usize) -> Event {
        record.start /= scale;
        record.end /= scale;
        Event {
            record: record,
            step: step / scale as usize,
            client: reqwest::Client::new(),
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
            end: 2000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.json".into(),
            },
        };

        let mut event = Event::new(record, 20, 1000);
        event.run().await?;
        for join in event.joins.into_iter() {
            tokio::join!(join);
        }
        mock.assert_hits(2);
        Ok(())
    }
}
