use crate::event::Event;
use crate::Record;
use tokio::task::JoinHandle;
use tracing::info;
use std::sync::Arc;

pub struct Generator {
    events: Vec<Event>,
    joins: Vec<JoinHandle<()>>,
}

impl Generator {
    pub fn from_records(records: Vec<Record>, step: usize, scale: u32) -> Generator {
        let mut events = Vec::new();
        for mut record in records.into_iter() {
            record.start /= scale;
            record.end /= scale;
            let event = Event::new(record, step / scale as usize);
            events.push(event);
        }
        events.sort();
        Generator {
            events,
            joins: Vec::new(),
        }
    }

    pub async fn start(&mut self) -> Result<(), crate::Error> {
        info!("Get ready to face your phobia :))");
        let mut current: u32 = 0;
        for mut event in self.events.clone().into_iter() {
            if current < event.record.start {
                let delay = (event.record.start - current) as u64;
                let duration = tokio::time::Duration::from_secs(delay);
                tokio::time::sleep(duration).await;
                current = event.record.start;
            }
            let join = tokio::spawn(async move {
                info!("Generator starting event: {:?}", event);
                let _ = event.run().await;
            });
            self.joins.push(join);
        }
        Ok(())
    }

    pub async fn start_unsafe(&'static mut self) -> Result<(), crate::Error> {
        info!("Get ready to face your phobia :))");
        let mut current: u32 = 0;
        for event in self.events.iter() {
            if current < event.record.start {
                let delay = (event.record.start - current) as u64;
                let duration = tokio::time::Duration::from_secs(delay);
                tokio::time::sleep(duration).await;
                current = event.record.start;
            }
            let event = Arc::new(event);
            let join = tokio::spawn(async move {
                info!("Generator starting event: {:?}", event); 
                match event.clone().run_unsafe().await {
                    Ok(_) => (),
                    Err(err) => info!("{:?} failed because {err}", event),
                }
            });
            self.joins.push(join);
        }
        Ok(())
    }

    pub async fn start_leak(&mut self) -> Result<(), crate::Error> {
        info!("Get ready to face your phobia :))");
        let mut current: u32 = 0;
        for mut event in self.events.clone().into_iter() {
            if current < event.record.start {
                let delay = (event.record.start - current) as u64;
                let duration = tokio::time::Duration::from_secs(delay);
                tokio::time::sleep(duration).await;
                current = event.record.start;
            }
            let join = tokio::spawn(async move {
                info!("Generator starting event: {:?}", event);
                let _ = event.run_leak().await;
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
            end: 2,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.yaml".into(),
                name: "file".into(),
            },
        };
        let record2 = crate::Record {
            method: "POST".into(),
            host: "http://localhost".into(),
            start: 1,
            end: 2,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.yaml".into(),
                name: "file".into(),
            },
        };
        let record3 = crate::Record {
            method: "POST".into(),
            host: "http://localhost".into(),
            start: 0,
            end: 1,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.yaml".into(),
                name: "file".into(),
            },
        };

        let records = vec![record2, record3.clone(), record1];
        let generator = Generator::from_records(records, 1, 1);

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
            end: 8,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.yaml".into(),
                name: "file".into(),
            },
        };
        let record2 = crate::Record {
            method: "POST".into(),
            host: server.base_url(),
            start: 2,
            end: 4,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.yaml".into(),
                name: "file".into(),
            },
        };
        let record3 = crate::Record {
            method: "POST".into(),
            host: server.base_url(),
            start: 0,
            end: 8,
            path: "/yolo/v2/predict".into(),
            body: crate::Body::MULTIPART {
                path: "./tests/data/test_data.yaml".into(),
                name: "file".into(),
            },
        };

        let records = vec![record2, record3, record1];
        let mut generator = Generator::from_records(records, 2, 2);

        generator.start().await?;
        generator.wait().await?;

        mock.assert_hits(9);
        Ok(())
    }
}
