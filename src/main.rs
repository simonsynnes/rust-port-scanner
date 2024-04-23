use std::{io::{self, Write}, net::{IpAddr, Ipv4Addr}, sync::mpsc::{channel, Sender}};
use bpaf::Bpaf;
use tokio::task;
use tokio::net::TcpStream;

const MAX_PORT_NUMBER: u16 = 65535;
const FALLBACK_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Arguments {
    #[bpaf(long("ip"), short('a'), argument("ip"), fallback(FALLBACK_IP))]
    pub ip: IpAddr,
    #[bpaf(long("start"), short('s'), guard(start_port_guard, "Must be greater than 0"), fallback(1u16))]
    pub from_port: u16,
    #[bpaf(long("end"), short('e'), guard(end_port_guard, "Must be less than or equal to 65535"), fallback(MAX_PORT_NUMBER))]
    pub to_port: u16,
}

fn start_port_guard(port: &u16) -> bool {
    *port > 0
}

fn end_port_guard(port: &u16) -> bool {
    *port > 0 && *port <= MAX_PORT_NUMBER
}

async fn scan(sender: Sender<u16>, ip: IpAddr, port: u16) {
    // formatting connection as IP:port
    match TcpStream::connect(format!("{}:{}", ip, port)).await {
        Ok(_) => {
            io::stdout().flush().unwrap();
            sender.send(port).unwrap();
        }
        Err(_) => {
        }
    }
}
#[tokio::main]
async fn main() {
    let args = arguments().run();
    let (sender, receiver) = channel();

    for port in args.from_port..args.to_port {
        let sender = sender.clone();
        task::spawn(async move {scan(sender, args.ip, port).await });
    }
    // drop connection and add ports to array
    drop(sender);
    let mut out = vec![];
    for p in receiver {
        out.push(p);
    }
    out.sort();
    
    if !out.is_empty() {
        for open_port in out {
            println!("{} is Open", open_port);
        }
    } else {
        println!("No open ports found");
    }
}
