use crate::dictionary;
use crate::global::Global;
use crate::options::Options;
use crate::scenario;
use diameter::transport::DiameterClient;
use diameter::transport::DiameterClientConfig;
use diameter::DiameterMessage;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio::task::LocalSet;
use tokio::time::{self, sleep, Duration};

// pub struct Runner {
//     param: RunParameter,
// }

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
        let duration_s = options.duration.as_secs() as u32;
        let total_iterations = rps * duration_s;

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
    let global = Global::new(&options.globals);

    let dict = dictionary::load(options.dictionaries.clone()).unwrap();
    let dict = Arc::new(dict);

    // TODO - remove hardcode
    let mut init_scenario = scenario::Scenario::new(
        options.scenarios.get(0).unwrap(),
        &global,
        Arc::clone(&dict),
    )
    .unwrap();

    // Skip first scenario, which is hardcoded as Init scenario for now
    let mut repeating_scenarios = vec![];
    for scenario in options.scenarios.iter().skip(1) {
        let s = scenario::Scenario::new(scenario, &global, Arc::clone(&dict)).unwrap();
        repeating_scenarios.push(s);
    }

    // let mut ccri_scenario = scenario::Scenario::new(
    //     options.scenarios.get(1).unwrap(),
    //     &global,
    //     Arc::clone(&dict),
    // )
    // .unwrap();

    // TODO
    // let mut ccrt_scenario =
    //     scenario::Scenario::new(options.scenarios.get(2).unwrap(), &global).unwrap();

    let local = LocalSet::new();
    local
        .run_until(async move {
            // Connect to server
            let config = DiameterClientConfig {
                use_tls: false,
                verify_cert: false,
            };
            let mut client = DiameterClient::new("localhost:3868", config);
            let mut handler = client.connect().await.unwrap();
            let dict_ref = Arc::clone(&dict);
            task::spawn_local(async move {
                DiameterClient::handle(&mut handler, dict_ref).await;
            });

            // Init scenario, send CER
            let cer = init_scenario.next_message().unwrap();
            if options.log_requests {
                log::info!("CER: {}", cer);
            }
            let resp = client.send_message(cer).await.unwrap();
            let cea = resp.await.unwrap();
            if options.log_responses {
                log::info!("CEA: {}", cea);
            }

            // Event Loop
            let (eventloop_tx, eventloop_rx) = channel(32);
            tokio::spawn(async move {
                event_loop(client, eventloop_rx).await.unwrap();
            });

            // Start Repeating Scenario
            log::info!(
                "Sending total request {} with {} TPS, batch size {}, interval {}",
                param.total_iterations,
                param.target_tps,
                param.batch_size,
                param.interval.as_secs_f64()
            );

            // let mut interval = time::interval(param.interval);

            let start = Instant::now();

            let (resp_tx, mut resp_rx) = channel(32);

            let ccri_scenario = repeating_scenarios.get_mut(0).unwrap();
            let ccri = ccri_scenario.next_message().unwrap();
            eventloop_tx
                .send(Event::SendMessage(ccri, resp_tx.clone()))
                .await
                .unwrap();

            if let Some(ccai) = resp_rx.recv().await {
                log::info!("Received CCA-I: {}", ccai);
            }

            let ccrt_scenario = repeating_scenarios.get_mut(1).unwrap();
            let ccrt = ccrt_scenario.next_message().unwrap();
            eventloop_tx
                .send(Event::SendMessage(ccrt, resp_tx.clone()))
                .await
                .unwrap();

            if let Some(ccat) = resp_rx.recv().await {
                log::info!("Received CCA-T: {}", ccat);
            }

            // for _ in 0..total_iterations {
            //     interval.tick().await;
            // }

            //
            // // We don't need atomic operation since we are running inside LocalSet
            // let counter: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
            //
            // for _ in 0..param.total_iterations / param.batch_size {
            //     interval.tick().await;
            //
            //     for _ in 0..param.batch_size {
            //         // let ccr = ccr(client.get_next_seq_num());
            //         let ccr = repeating_scenario.next_message().unwrap();
            //         if options.log_requests {
            //             log::info!("Request: {}", ccr);
            //         }
            //
            //         let counter = Rc::clone(&counter);
            //         let resp = client.send_message(ccr).await.unwrap();
            //         let _ = task::spawn_local(async move {
            //             let cca = resp.await.expect("Failed to get response");
            //             if options.log_responses {
            //                 log::info!("Response: {}", cca);
            //             }
            //             *counter.borrow_mut() += 1;
            //         });
            //     }
            // }
            //
            // log::info!("Waiting for all requests to finish");
            // while *counter.borrow() < param.total_iterations {
            //     sleep(Duration::from_millis(50)).await;
            // }
            //

            // sleep 1
            // sleep(Duration::from_secs(1)).await;

            // Terminate the event loop
            eventloop_tx.send(Event::Terminate).await.unwrap();

            let elapsed = start.elapsed();
            let elapsed_s = elapsed.as_secs() as f64 + elapsed.subsec_millis() as f64 / 1000.0;
            let tps = param.total_iterations as f64 / (elapsed.as_micros() as f64 / 1_000_000.0);
            log::info!("Elapsed: {:.3}s , {} requests per second", elapsed_s, tps,);

            RunReport { tps, elapsed }
        })
        .await
}

// struct EventContext {
//     scenario_id: usize,
// }

enum Event {
    SendMessage(DiameterMessage, Sender<DiameterMessage>),
    Terminate,
}

async fn event_loop(
    mut client: DiameterClient,
    mut rx: Receiver<Event>,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(event) = rx.recv().await {
        match event {
            Event::SendMessage(request, tx) => {
                log::info!("Sending message: {}", request);
                // send message
                let resp = client.send_message(request).await.unwrap();
                let response = resp.await.unwrap();

                // log::info!("Received response: {}", response);

                // Send response back to main runner loop
                tx.send(response).await.unwrap();
            }
            Event::Terminate => {
                log::info!("Terminating event loop");
                break;
            }
        }
    }
    todo!()
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
            call_timeout: Duration::from_millis(2000),
            duration: Duration::from_secs(120),
            log_requests: false,
            log_responses: false,
            protocol: options::Protocol::Diameter,
            globals: options::Global { variables: vec![] },
            dictionaries: vec![],
            scenarios: vec![],
        };

        let param = RunParameter::new(&options);

        assert_eq!(param.batch_size, 2);
        assert_eq!(param.interval.as_secs_f64(), 0.004);
        assert_eq!(param.total_iterations, 60000);
    }
}
