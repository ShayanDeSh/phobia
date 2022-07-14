use phobia::Record;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::debug;

fn read_data(path: PathBuf) -> Result<Vec<Record>, phobia::Error> {
    let config_file = std::fs::File::open(path).ok();
    if let Some(file) = config_file {
        let config = serde_json::from_reader(file)?;
        return Ok(config);
    }
    Err("Could not open file".into())
}

#[tokio::main]
async fn main() -> Result<(), phobia::Error> {
    tracing_subscriber::fmt::try_init()?;

    let cmd = CMD::from_args();

    debug!("{:?}", cmd);
    let data = read_data(cmd.path)?;

    debug!("{:?}", data);

    Ok(())
}

#[derive(StructOpt, Serialize, Deserialize, Debug)]
#[structopt(name = "phobia")]
struct CMD {
    #[structopt(short, long)]
    concurency: u8,
    #[structopt(long)]
    scale: u8,
    #[structopt(short, long)]
    step: u32,
    #[structopt(parse(from_os_str))]
    path: PathBuf,
}

#[cfg(test)]
mod tests {
    use crate::read_data;

    #[test]
    fn test_read_data() -> Result<(), phobia::Error> {
        let data = read_data("./tests/data/test_data.json".into())?;
        println!("{:?}", data);
        assert_eq!(data.len(), 2);
        Ok(())
    }
}
