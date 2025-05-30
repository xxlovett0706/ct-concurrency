use std::{thread, time::Duration};

use concurrency::AmapMetrics;
use rand::Rng;

const N: usize = 2;
const M: usize = 4;

fn main() {
    let metrics = AmapMetrics::new(&[
        "call.thread.worker.0",
        "call.thread.worker.1",
        "req.page.1",
        "req.page.2",
        "req.page.3",
        "req.page.4",
        "req.page.5",
    ]);

    // start N workers and M requesters
    for i in 0..N {
        task_worker(i, metrics.clone());
    }
    for _ in 0..M {
        request_worker(metrics.clone());
    }

    loop {
        thread::sleep(Duration::from_millis(5000));
        println!("{}", metrics);
    }
}

fn task_worker(idx: usize, metrics: AmapMetrics) {
    thread::spawn(move || loop {
        let mut rng = rand::rng();
        thread::sleep(Duration::from_millis(rng.random_range(100..=5000)));
        if let Err(e) = metrics.inc(format!("call.thread.worker.{}", idx)) {
            eprintln!("Error incrementing metrics: {}", e);
        }
    });
}

fn request_worker(metrics: AmapMetrics) {
    thread::spawn(move || loop {
        let mut rng = rand::rng();
        thread::sleep(Duration::from_millis(rng.random_range(50..=800)));

        let page = rng.random_range(1..=5);
        if let Err(e) = metrics.inc(format!("req.page.{}", page)) {
            eprintln!("Error incrementing metrics: {}", e);
        }
    });
}
