use clap::Parser;
use std::sync::{Arc};
use std::net::{Ipv4Addr};
use std::collections::{HashMap};
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::{Mutex, oneshot, Semaphore, OwnedSemaphorePermit};
use tokio::time::{sleep, Duration};


#[derive(Debug, Parser)]
struct Opt {
  #[clap(short, long, default_value="127.0.0.1:9191")]
  target: String,
  #[clap(short, long, default_value="1000")]
  max_concurrent: u32,
  #[clap(short='T', long, default_value="1000000")]
  total: u32,
  #[clap(short, long, default_value="out.csv")]
  output: String,
}

struct AccountItem {
  sent_at: Instant,
  received_at: Option<Instant>,
  permit: Option<OwnedSemaphorePermit>,
}

impl AccountItem {
  fn new(sent_at: Instant, permit: OwnedSemaphorePermit) -> Self {
    Self {
      sent_at,
      received_at: None,
      permit: Some(permit),
    }
  }
}

#[tokio::main]
async fn main() {
  let opt = Opt::parse();

  let total = opt.total;

  let target_addr = opt.target.parse::<std::net::SocketAddr>().unwrap();

  let accounting: Arc<Mutex<HashMap<u32, AccountItem>>> = Arc::new(Mutex::new(HashMap::new()));

  let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();

  let socket = Arc::new(socket);

  let socket1 = socket.clone();
  let accounting1 = accounting.clone();

  // use a Semaphore to limit number of concurrent requests
  let semaphore = Arc::new(Semaphore::new(opt.max_concurrent as usize));

  let (tx, mut rx) = oneshot::channel();

  tokio::spawn(async move {
    let mut current_id = 0;

    for _ in 0..total {
      let now = Instant::now();
      let id = current_id;

      current_id += 1;

      let permit = semaphore.clone().acquire_owned().await.unwrap();

      accounting.lock().await.insert(id, AccountItem::new(now, permit));

      let bytes: [u8;4] = u32::to_ne_bytes(id.to_be());

      // println!("Sending: {}, {:?}", id, bytes);

      socket.send_to(&bytes, target_addr).await.unwrap();
    }

    println!("Send loop done.");

    // wait for 10 seconds and then signal kill
    sleep(Duration::from_secs(3)).await;

    let _ = tx.send(());
  });

  let mut buffer: [u8; 4] = [0; 4];

  let mut n_received = 0;

  'outer: loop {
    tokio::select! {
      result = socket1.recv_from(&mut buffer)  => {
        if let Ok((_len, _addr)) = result {
          let id = u32::from_be_bytes(buffer);

          accounting1.lock().await.entry(id).and_modify(|x| {
            if let Some(_) = x.permit.take() {
              let now = Instant::now();
              x.received_at = Some(now);
              x.permit = None;

            // println!("Received: {}, {}, {:?}, {}", id, len, buffer, now.duration_since(x.sent_at).as_nanos());

            n_received += 1;
            }
          });

          // if n_received % 1000 == 0 {
          //   println!("Received {} packets", n_received);
          // }
        }
      }
      _ = &mut rx => {
        break 'outer;
      }
    }
  };

  println!("Received back {} packets", n_received);

  let mut durations: Vec<u128> = accounting1.lock().await.iter().filter_map(|(_, value)| {
    match value.received_at {
      Some(received_at) => Some(received_at.duration_since(value.sent_at).as_nanos()),
      None => None,
    }
  }).collect();

  durations.sort();

  let mean: f64 = durations.iter().sum::<u128>() as f64 / durations.len() as f64;

  let max = *durations.iter().max().unwrap();

  let min =  *durations.iter().min().unwrap();

  let median = durations[durations.len() / 2];

  let q10 = durations[durations.len() / 10];

  let q90 = durations[durations.len() * 9 / 10];

  let var = durations.iter().map(|x| ((*x as f64) - mean).powf(2.0)).sum::<f64>() / durations.len() as f64;

  let std = var.sqrt();

  println!("mean: {}, median: {}, q10: {}, q90: {}, max: {}, min: {}, var: {}, std: {}",
    mean, median, q10, q90, max, min, var, std);
  
  let mut wtr = csv::Writer::from_path(opt.output).unwrap();
  wtr.write_record(&["id", "sent_at", "received_at", "rtt_nanos"]).unwrap();

  for (id, item) in accounting1.lock().await.iter() {
    let rtt_nanos = match item.received_at {
      Some(received_at) => Some(received_at.duration_since(item.sent_at).as_nanos()),
      None => None,
    };
    wtr.serialize((id, item.sent_at.elapsed().as_nanos(), item.received_at.map(|r| r.elapsed().as_nanos()), rtt_nanos)).unwrap();
  }

  wtr.flush().unwrap();
}