use std::{io::stdin, net::SocketAddr, str::FromStr};

use async_trait::async_trait;
use clap::Parser;
use tabled::{Table, Tabled};
use turn_driver::{start_hooks_server, Controller, Events, Hooks, QueryFilter, Transport};

struct HooksImpl;

#[async_trait]
impl Hooks for HooksImpl {
    async fn auth(
        &self,
        addr: SocketAddr,
        name: String,
        realm: String,
        rid: String,
    ) -> Option<&str> {
        println!(
            "auth: addr={}, name={}, realm={}, rid={}",
            addr, name, realm, rid
        );

        Some("test")
    }

    async fn on(&self, event: Events, realm: String, rid: String) {
        println!("event={:?}, realm={}, rid={}", event, realm, rid)
    }
}

struct Repl;

impl Iterator for Repl {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        println!("\nPlease enter the address or username to query session information:");

        let mut input = String::new();
        if let Ok(_) = stdin().read_line(&mut input) {
            Some(input.replace("\n", ""))
        } else {
            None
        }
    }
}

#[derive(Tabled)]
struct BaseInfo {
    software: String,
    uptime: u64,
    port_allocated: u16,
    port_capacity: u16,
}

#[derive(Tabled)]
struct Interface {
    transport: String,
    bind: SocketAddr,
    external: SocketAddr,
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    bind: SocketAddr,
    #[arg(long)]
    server: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let controller = Controller::new(&cli.server);
    tokio::spawn(start_hooks_server(cli.bind, HooksImpl));

    if let Some(info) = controller.get_info().await {
        println!("Base info:");
        println!(
            "{}\r\n",
            Table::new([BaseInfo {
                software: info.payload.software,
                uptime: info.payload.uptime,
                port_allocated: info.payload.port_allocated,
                port_capacity: info.payload.port_capacity,
            }])
            .to_string()
        );

        println!("Interfaces:");
        println!(
            "{}",
            Table::new(
                info.payload
                    .interfaces
                    .into_iter()
                    .map(|it| Interface {
                        transport: if it.transport == Transport::UDP {
                            "UDP"
                        } else {
                            "TCP"
                        }
                        .to_string(),
                        external: it.external,
                        bind: it.bind,
                    })
                    .collect::<Vec<Interface>>()
            )
            .to_string()
        );
    } else {
        println!("turn server not runing!");
        return;
    }

    for input in Repl {
        let query = SocketAddr::from_str(&input)
            .map(|it| QueryFilter::Addr(it))
            .unwrap_or_else(|_| QueryFilter::UserName(&input));

        if let Some(session) = controller.get_session(&query).await {
            println!("\r\nSessions:");
            println!("{:#?}", session.payload);
        }
    }
}
