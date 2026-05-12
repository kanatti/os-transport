mod protocol;
mod capture;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "os-transport", about = "OpenSearch transport protocol parser")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Decode a single message from a hex string
    Decode {
        /// Hex string (spaces allowed)
        hex: String,
    },

    /// Read and parse all messages from a pcap file
    Read {
        /// Path to pcap file
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Decode { hex } => cmd_decode(hex),
        Command::Read { path } => cmd_read(path),
    }
}

fn cmd_decode(hex_str: String) {
    let clean: String = hex_str.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.len() % 2 != 0 {
        eprintln!("error: hex string must have even number of digits");
        std::process::exit(1);
    }
    let data: Vec<u8> = (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).unwrap())
        .collect();

    match protocol::message::Message::parse(&data) {
        Ok((msg, consumed)) => {
            print_message(&msg);
            println!("  Consumed: {} bytes", consumed);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_read(path: PathBuf) {
    let messages = capture::pcap::read_pcap(&path).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(1);
    });

    for pm in &messages {
        print!(
            "{} → {}  ",
            format_endpoint_short(&pm.src),
            format_endpoint_short(&pm.dst),
        );
        print_message(&pm.message);
    }

    eprintln!("\n--- {} messages ---", messages.len());
}

fn print_message(msg: &protocol::message::Message) {
    println!("{}", msg);
    if !msg.var_header.request_headers.is_empty() {
        for (k, v) in &msg.var_header.request_headers {
            println!("    {}: {}", k, v);
        }
    }
}

fn format_endpoint_short(ep: &capture::reassembly::Endpoint) -> String {
    format!(":{}", ep.port)
}
