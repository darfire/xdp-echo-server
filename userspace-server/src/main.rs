use tokio;
use tokio::net::UdpSocket;
use clap::Parser;

#[derive(Debug, Parser)]
struct Opt {
  #[clap(short, long, default_value="127.0.0.1:9191")]
  address: String,
}


#[tokio::main]
async fn main() {
  let opt = Opt::parse();

  let sock = UdpSocket::bind(opt.address).await.unwrap();

  let mut buf: [u8; 4] = [0; 4];

  println!("Listening on: {}", sock.local_addr().unwrap());

  // let mut n_received = 0;
  
  loop {
    let (len, addr) = sock.recv_from(&mut buf).await.unwrap();

    // n_received += 1;

    // if n_received % 1000 == 0 {
    //   println!("Received {} packets. Last addr {}", n_received, addr);
    // }

    // send it back right away
    sock.send_to(&buf[..len], &addr).await.unwrap();

    
  }
}