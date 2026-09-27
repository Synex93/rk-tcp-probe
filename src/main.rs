use std::{
    io,
    net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs},
    thread,
    time::{Duration, Instant},
};

const HOST: &str = "www.baidu.com";
const PORT: u16 = 443;

const INTERVAL: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

// 连接成功后保持一段时间，便于在 /proc/net/tcp 中观察。
// 期间不发送任何应用层数据。
const HOLD_TIME: Duration = Duration::from_secs(1);

fn resolve_ipv4() -> io::Result<Vec<SocketAddr>> {
    let addresses: Vec<SocketAddr> = (HOST, PORT)
        .to_socket_addrs()?
        .filter(|addr| addr.is_ipv4())
        .collect();

    if addresses.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "没有解析到 IPv4 地址",
        ));
    }

    Ok(addresses)
}

fn main() -> io::Result<()> {
    let addresses = resolve_ipv4()?;

    println!("target: {HOST}:{PORT}");
    println!("resolved IPv4 addresses: {addresses:?}");
    println!("interval: {:?}, hold: {:?}", INTERVAL, HOLD_TIME);

    let mut address_index = 0usize;
    let mut next_start = Instant::now();

    loop {
        let address = addresses[address_index % addresses.len()];
        address_index += 1;

        let started_at = Instant::now();

        match TcpStream::connect_timeout(&address, CONNECT_TIMEOUT) {
            Ok(stream) => {
                println!("[+] TCP connected to {address}");

                // 不调用 read/write，不发送 HTTP 或 TLS 数据。
                thread::sleep(HOLD_TIME);

                let _ = stream.shutdown(Shutdown::Both);
                println!("[-] TCP connection closed");
            }

            Err(error) => {
                eprintln!("[!] TCP connect to {address} failed: {error}");
            }
        }

        // 尽量让每次连接的开始时间间隔接近 5 秒。
        next_start += INTERVAL;

        let now = Instant::now();
        if next_start > now {
            thread::sleep(next_start - now);
        } else {
            // 如果连接耗时超过了间隔，则从当前时间重新计时。
            next_start = Instant::now();
        }

        // 防止极端情况下 started_at 被优化掉，也方便调试时保留时间变量。
        let _elapsed = started_at.elapsed();
    }
}
