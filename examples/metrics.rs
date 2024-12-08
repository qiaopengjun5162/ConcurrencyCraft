use std::{thread, time::Duration};

use anyhow::Result;
use concurrency::Metrics;
use rand::Rng;

const N: usize = 2;
const M: usize = 4;

fn main() -> Result<()> {
    let metrics = Metrics::new();
    // for i in 0..100 {
    //     metrics.inc("req.page.1");
    //     metrics.inc("req.page.2");
    //     if i % 2 == 0 {
    //         metrics.inc("req.page.3");
    //     }
    // }
    // for _ in 0..27 {
    //     metrics.inc("call.thread.worker.1");
    // }

    println!("metrics: {:?}", metrics.snapshot());

    for idx in 0..N {
        task_worker(idx, metrics.clone()); // Metrics {data: Arc::clone(&metrics.data)}
    }

    for _ in 0..M {
        request_worker(metrics.clone());
    }
    loop {
        thread::sleep(Duration::from_secs(2));
        println!("metrics: {:?}", metrics.snapshot());
    }
}

fn task_worker(idx: usize, metrics: Metrics) {
    thread::spawn(move || loop {
        // do long term stuff
        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
        metrics.inc(format!("call.thread.worker.{}", idx)).unwrap();
    });
}

fn request_worker(metrics: Metrics) {
    thread::spawn(move || loop {
        // do long term stuff
        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_millis(rng.gen_range(50..800)));
        let page = rng.gen_range(1..256);
        metrics.inc(format!("req.page.{}", page)).unwrap();
    });
}
