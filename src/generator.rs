use std::borrow::BorrowMut;

use crate::Record;
use crate::event::Event;

pub struct Generator {
    events: Vec<Event>
}

impl Generator {
    pub fn from_records(records: Vec<Record>, step: usize, scale: u32) -> Generator {
        let mut events = Vec::new();
        for record in records.into_iter() {
            let event = Event::new(record, scale, step);
            events.push(event);
        }
        events.sort();
        Generator { events }
    }

    pub async fn start(self) -> Result<(), crate::Error> {
        let mut current: u32 = 0;
        for mut event in self.events.into_iter() {
            if current < event.record.start {
                let delay = (event.record.start - current) as u64;
                let duration = tokio::time::Duration::from_millis(delay);
                tokio::time::sleep(duration).await;
                current = event.record.start;
            }
            tokio::spawn(async move {
                let _ = event.borrow_mut().run().await;
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod generator_tests {
    use crate::generator::Generator;

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
        
        let records = vec![record2.clone(), record3.clone(), record1.clone()];
        let generator = Generator::from_records(records, 1000, 1);

        assert!(generator.events.get(0).unwrap().record.end == record3.end);
    }
}
