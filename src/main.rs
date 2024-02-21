mod global;
mod options;
mod scenario;

use crate::global::Global;
use crate::options::Options;
use chrono::Local;
use diameter::transport::eventloop::DiameterClient;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::thread;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task;
use tokio::task::LocalSet;
use tokio::time::{self, sleep, Duration};

// static COUNTER: AtomicU32 = AtomicU32::new(0);

#[tokio::main]
// #[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::Builder::new()
        .format(|buf, record| {
            let now = Local::now();
            let thread = thread::current();
            let thread_name = thread.name().unwrap_or("unnamed");
            let thread_id = thread.id();

            writeln!(
                buf,
                "{} [{}] {:?} - ({}): {}",
                now.format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                thread_id,
                thread_name,
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Info)
        .init();

    let (tx, mut rx) = mpsc::channel(8);

    let options = options::load("./options.lua");

    for _ in 0..options.parallel {
        let tx = tx.clone();
        let options = options.clone();
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let report = run(options).await;
                tx.send(report).await.unwrap();
            });
        });
    }

    drop(tx);

    let mut total_tps = 0f64;
    while let Some(report) = rx.recv().await {
        total_tps += report.tps;
    }
    log::info!("Total TPS: {}", total_tps);
}

pub struct Report {
    pub tps: f64,
}

async fn run(options: Options) -> Report {
    let local = LocalSet::new();
    let rps = options.call_rate;
    let (batch_size, interval, total_iterations) = calc_batch_interval(&options);

    log::debug!("Options is {:?}", options);
    log::info!(
        "Sending {} requests per second with batch size {}, interval {}",
        rps,
        batch_size,
        interval.as_secs_f64()
    );
    log::info!("Total iterations: {}", total_iterations);

    let mut interval = time::interval(interval);

    let global = Global::new(&options.globals);
    let mut scenario = scenario::Scenario::new(&options, &global).unwrap();

    local
        .run_until(async move {
            // Connect to server
            let mut client = DiameterClient::new("localhost:3868");
            let _ = client.connect().await;

            // Start time
            let start = Instant::now();

            // We don't need atomic operation since we are running inside LocalSet
            let counter: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

            for _ in 0..total_iterations / batch_size {
                interval.tick().await;

                for _ in 0..batch_size {
                    // let ccr = ccr(client.get_next_seq_num());
                    let ccr = scenario.next_message().unwrap();
                    if options.log_requests {
                        log::info!("Request: {}", ccr);
                    }

                    let counter = Rc::clone(&counter);
                    let mut request = client.request(ccr).await.unwrap();
                    let _ = task::spawn_local(async move {
                        let _ = request.send().await.expect("Failed to create request");
                        let _cca = request.response().await.expect("Failed to get response");
                        if options.log_responses {
                            log::info!("Response: {}", _cca);
                        }
                        *counter.borrow_mut() += 1;
                    });
                }
            }
            log::info!("Waiting for all requests to finish");

            while *counter.borrow() < total_iterations {
                sleep(Duration::from_millis(50)).await;
            }

            let elapsed = start.elapsed();
            let tps = total_iterations as f64 / (elapsed.as_micros() as f64 / 1_000_000.0);
            log::info!(
                "Elapsed: {}.{}s , {} requests per second",
                elapsed.as_secs(),
                elapsed.subsec_micros(),
                tps,
            );

            Report { tps }
        })
        .await
}

fn calc_batch_interval(options: &Options) -> (u32, Duration, u32) {
    let rps = options.call_rate;
    let batch_size = (rps / 200) as u32;
    let batch_size = if batch_size == 0 { 1 } else { batch_size };
    let batches_per_second = rps as f64 / batch_size as f64;
    let interval = Duration::from_secs_f64(1.0 / batches_per_second);
    let total_iterations = rps * options.duration_s;

    return (batch_size, interval, total_iterations);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_calculate() {
        let options = Options {
            parallel: 1,
            call_rate: 500,
            call_timeout_ms: 2000,
            duration_s: 120,
            log_requests: false,
            log_responses: false,
            globals: options::Global { variables: vec![] },
            scenarios: vec![],
        };

        let (batch_size, interval, total_iterations) = calc_batch_interval(&options);

        assert_eq!(batch_size, 2);
        assert_eq!(interval.as_secs_f64(), 0.004);
        assert_eq!(total_iterations, 60000);
    }
}
