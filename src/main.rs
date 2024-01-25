use chrono::Local;
use diameter::avp;
use diameter::avp::enumerated::EnumeratedAvp;
use diameter::avp::identity::IdentityAvp;
use diameter::avp::unsigned32::Unsigned32Avp;
use diameter::avp::utf8string::UTF8StringAvp;
use diameter::avp::Avp;
use diameter::client::DiameterClient;
use diameter::diameter::{ApplicationId, CommandCode, DiameterMessage, REQUEST_FLAG};
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Instant;
use tokio::time::{self, sleep, Duration};

static COUNTER: AtomicU32 = AtomicU32::new(0);

// #[tokio::main]
#[tokio::main(flavor = "current_thread")]
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

    // let batch_size = 8000;
    // let rps = 180000;
    // let total_iterations = 1800000;
    let batch_size = 1;
    let rps = 1;
    let total_iterations = 3;
    let batches_per_second = rps as f64 / batch_size as f64;
    let interval = Duration::from_secs_f64(1.0 / batches_per_second);

    log::info!(
        "Sending {} requests per second with batch size {}, interval {}",
        rps,
        batch_size,
        interval.as_secs_f64()
    );

    let mut interval = time::interval(interval);

    let mut client = DiameterClient::new("localhost:3868");
    let _ = client.connect().await;

    // Start time
    let start = Instant::now();

    // Fire Requests
    let seq_id = AtomicU32::new(0);

    for _ in 0..total_iterations / batch_size {
        interval.tick().await;

        for _ in 0..batch_size {
            let seq_id = seq_id.fetch_add(1, Ordering::SeqCst);
            let ccr = ccr(seq_id);
            log::info!("Request: {}", ccr);
            let mut request = client.request(ccr).await.unwrap();
            let _handle = tokio::spawn(async move {
                let _ = request.send().await.expect("Failed to create request");
                let _cca = request.response().await.expect("Failed to get response");
                log::info!("Response: {}", _cca);
                COUNTER.fetch_add(1, Ordering::SeqCst);
            });
        }
    }

    while COUNTER.load(Ordering::Relaxed) < total_iterations {
        sleep(Duration::from_millis(50)).await;
    }

    let elapsed = start.elapsed();
    log::info!(
        "Elapsed: {}.{}s , {} requests per second",
        elapsed.as_secs(),
        elapsed.subsec_micros(),
        total_iterations as f64 / (elapsed.as_micros() as f64 / 1_000_000.0)
    );
}

pub fn ccr(seq_id: u32) -> DiameterMessage {
    let mut ccr = DiameterMessage::new(
        CommandCode::CreditControl,
        ApplicationId::CreditControl,
        REQUEST_FLAG,
        seq_id,
        seq_id,
    );
    ccr.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
    ccr.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
    ccr.add_avp(avp!(263, None, UTF8StringAvp::new("ses;12345888"), true));
    ccr.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
    ccr.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
    ccr
}
