use anyhow::{Ok, Result};
use rand::Rng;
use std::{sync::mpsc, thread, time::Duration};

const NUM_PRODUCERS: usize = 4;

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    idx: usize,
    value: u64,
}

impl Msg {
    fn new(idx: usize, value: u64) -> Self {
        Self { idx, value }
    }
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // 创建producer
    for i in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(i, tx));
    }

    // drop(tx);

    // 创建consumer
    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("consumer: {:?}", msg);
        }

        println!("consumer exit");
        42
    });

    let secret = consumer
        .join()
        .map_err(|e| anyhow::anyhow!("Thread join error: {:?}", e))?;

    println!("secret: {}", secret);

    Ok(())
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let mut rng = rand::rng();
        let value = rng.random::<u64>();
        tx.send(Msg::new(idx, value))?;

        let sleep_time = rng.random::<u8>() as u64 * 10;
        thread::sleep(Duration::from_millis(sleep_time));

        // random exit
        if rng.random::<u8>() % 10 == 0 {
            println!("producer {} exit", idx);
            break;
        }
    }

    Ok(())
}
