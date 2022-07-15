use crate::event::Event;
use crate::Record;
use tokio::task::JoinHandle;

pub struct Generator {
    events: Vec<Event>,
    joins: Vec<JoinHandle<()>>,
}

impl Generator {
    pub fn from_records(records: Vec<Record>, step: usize, scale: u32) -> Generator {
        let mut events = Vec::new();
        for record in records.into_iter() {
            let event = Event::new(record, scale, step);
            events.push(event);
        }
        events.sort();
        Generator {
            events,
            joins: Vec::new(),
        }
    }

    pub async fn start(&mut self) -> Result<(), crate::Error> {
        let mut current: u32 = 0;
        for mut event in self.events.clone().into_iter() {
            if current < event.record.start {
                let delay = (event.record.start - current) as u64;
                let duration = tokio::time::Duration::from_millis(delay);
                tokio::time::sleep(duration).await;
                current = event.record.start;
            }
            let join = tokio::spawn(async move {
                let _ = event.run().await;
                let _ = event.wait().await;
            });
            self.joins.push(join);
        }
        Ok(())
    }

    pub async fn wait(self) -> Result<(), crate::Error> {
        for join in self.joins.into_iter() {
            tokio::join!(join).0?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod generator_tests {
    use crate::generator::Generator;
    use httpmock::prelude::*;

    #[test]
    fn test_from_records() {
        let record1 = crate::Record {
            method: "POST".into(),
            host: "http://localhost".into(),
            start: 0,
            end: 2000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.json".into(),
            },
        };
        let record2 = crate::Record {
            method: "POST".into(),
            host: "http://localhost".into(),
            start: 1000,
            end: 2000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.json".into(),
            },
        };
        let record3 = crate::Record {
            method: "POST".into(),
            host: "http://localhost".into(),
            start: 0,
            end: 1000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.json".into(),
            },
        };

        let records = vec![record2, record3.clone(), record1];
        let generator = Generator::from_records(records, 1000, 1);

        assert!(generator.events.get(0).unwrap().record.end == record3.end);
    }

    #[tokio::test]
    async fn test_generator() -> Result<(), crate::Error> {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/yolo/v2/predict")
                .body_contains("file");
            then.status(200);
        });

        let record1 = crate::Record {
            method: "POST".into(),
            host: server.base_url(),
            start: 0,
            end: 2000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.json".into(),
            },
        };
        let record2 = crate::Record {
            method: "POST".into(),
            host: server.base_url(),
            start: 1000,
            end: 2000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.json".into(),
            },
        };
        let record3 = crate::Record {
            method: "POST".into(),
            host: server.base_url(),
            start: 0,
            end: 1000,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.json".into(),
            },
        };

        let records = vec![record2, record3, record1];
        let mut generator = Generator::from_records(records, 1000, 20);

        generator.start().await?;
        generator.wait().await?;

        mock.assert_hits(4);
        Ok(())
    }
}
