use crate::global::Global;
use crate::options::Options;
use crate::scenario;
use diameter::transport::eventloop::DiameterClient;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tokio::task;
use tokio::task::LocalSet;
use tokio::time::{self, sleep, Duration};

#[derive(Clone)]
pub struct RunParameter {
    pub target_tps: u32,
    pub batch_size: u32,
    pub interval: Duration,
    pub total_iterations: u32,
}

impl RunParameter {
    pub fn new(options: &Options) -> RunParameter {
        let rps = options.call_rate;
        let batch_size = (rps / 200) as u32;
        let batch_size = if batch_size == 0 { 1 } else { batch_size };
        let batches_per_second = rps as f64 / batch_size as f64;
        let interval = Duration::from_secs_f64(1.0 / batches_per_second);
        let total_iterations = rps * options.duration_s;

        RunParameter {
            target_tps: rps,
            batch_size,
            interval,
            total_iterations,
        }
    }
}

pub struct RunReport {
    pub tps: f64,
    pub elapsed: Duration,
}

pub async fn run(options: Options, param: RunParameter) -> RunReport {
    log::info!(
        "Sending total request {} with {} TPS, batch size {}, interval {}",
        param.total_iterations,
        param.target_tps,
        param.batch_size,
        param.interval.as_secs_f64()
    );

    let mut interval = time::interval(param.interval);

    let global = Global::new(&options.globals);
    let mut scenario = scenario::Scenario::new(&options, &global).unwrap();

    let local = LocalSet::new();
    local
        .run_until(async move {
            // Connect to server
            let mut client = DiameterClient::new("localhost:3868");
            let _ = client.connect().await;

            // Start time
            let start = Instant::now();

            // We don't need atomic operation since we are running inside LocalSet
            let counter: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

            for _ in 0..param.total_iterations / param.batch_size {
                interval.tick().await;

                for _ in 0..param.batch_size {
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

            while *counter.borrow() < param.total_iterations {
                sleep(Duration::from_millis(50)).await;
            }

            let elapsed = start.elapsed();
            let tps = param.total_iterations as f64 / (elapsed.as_micros() as f64 / 1_000_000.0);
            log::info!(
                "Elapsed: {}.{}s , {} requests per second",
                elapsed.as_secs(),
                elapsed.subsec_micros(),
                tps,
            );

            RunReport { tps, elapsed }
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options;

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

        let param = RunParameter::new(&options);

        assert_eq!(param.batch_size, 2);
        assert_eq!(param.interval.as_secs_f64(), 0.004);
        assert_eq!(param.total_iterations, 60000);
    }
}
