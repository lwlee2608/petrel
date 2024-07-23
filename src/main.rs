mod dictionary;
mod global;
mod options;
mod runner;
mod scenario;

use chrono::Local;
use std::io::Write;
use std::thread;
use tokio::sync::mpsc;

#[tokio::main]
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

    // Load Config file
    let options = options::load("./options.lua");
    log::debug!("Options is {:?}", options);

    // Runners
    let (tx, mut rx) = mpsc::channel(8);
    for _ in 0..options.parallel {
        let tx = tx.clone();
        let options = options.clone();
        let param = runner::RunParameter::new(&options);
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let report = runner::run(options, param).await;
                tx.send(report).await.unwrap();
            });
        });
    }

    drop(tx);

    let mut total_tps = 0f64;
    let mut elapsed = tokio::time::Duration::from_secs(0);
    while let Some(report) = rx.recv().await {
        total_tps += report.tps;
        elapsed = elapsed.max(report.elapsed);
    }

    log::info!("Total TPS: {}", total_tps);
    log::info!("Elapsed: {:?}", elapsed);
}
