use crate::Record;
use std::borrow::Cow;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::task::JoinHandle;

pub struct Event {
    pub record: Record,
    step: usize,
    scale: u32,
    joins: Vec<JoinHandle<()>>,
}

impl Event {
    pub async fn run(&mut self) -> Result<(), crate::Error> {
        match &self.record.body {
            crate::Body::MULTIPART { path } => {
                let mut file = File::open(path).await?;
                let mut contents = vec![];
                file.read_to_end(&mut contents).await?;
                for _ in (self.record.start..self.record.end).step_by(self.step) {
                    let c = contents.clone();
                    let r = self.record.clone();
                    let join = tokio::spawn(async move {
                        let _response = Self::send_file(r, c.into()).await;
                    });
                    tokio::time::sleep(tokio::time::Duration::from_millis(self.step as u64)).await;
                    self.joins.push(join);
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
            end: 2000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.json".into(),
            },
        };

        let mut event = Event::new(record, 20, 1000);
        event.run().await?;
        for join in event.joins.into_iter() {
            let _ = tokio::join!(join);
        }
        mock.assert_hits(2);
        Ok(())
    }
}
