//! HTTP Request Tracer & Module Energy Attributor in Pure Rust
//! Mengukur konsumsi energi per-request HTTP menggunakan ProcessEnergyEngine

use crate::engine::ProcessEnergyEngine;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTraceRecord {
    pub endpoint: String,
    pub method: String,
    pub status: u16,
    pub latency_ms: f64,
    pub cpu_active_time_s: f64,
    pub energy_total_joules: f64,
    pub co2e_grams: f64,
    pub cpu_utilization: f64,
}

pub struct ModuleAttributor<'a> {
    pub engine: &'a ProcessEnergyEngine,
    pub records: Vec<RequestTraceRecord>,
}

impl<'a> ModuleAttributor<'a> {
    pub fn new(engine: &'a ProcessEnergyEngine) -> Self {
        Self {
            engine,
            records: Vec::new(),
        }
    }

    pub fn trace_http_request(
        &mut self,
        url: &str,
        method: &str,
        _headers: &[(String, String)],
        body: Option<&str>,
    ) -> RequestTraceRecord {
        // 1. Ambil snapshot CPU sebelum request
        let start_snap = self.engine.capture_snapshot();
        let start_time = Instant::now();

        // 2. Eksekusi HTTP Request via Native TcpStream (Zero external C libraries)
        let (status, wall_clock_sec) = execute_http_request(url, method, body, start_time);
        let latency_ms = wall_clock_sec * 1000.0;

        // 3. Ambil snapshot CPU setelah request selesai
        let end_snap = self.engine.capture_snapshot();

        // 4. Hitung energi bersih murni (Kernel Process-Level Times)
        let (cpu_active, energy_joules, cpu_util) =
            self.engine.calculate_energy(&start_snap, &end_snap, wall_clock_sec);

        let energy_kwh = energy_joules / 3_600_000.0;
        let co2e_grams = energy_kwh * 850.0; // Grid intensity factor 850 gCO2e/kWh

        let record = RequestTraceRecord {
            endpoint: url.to_string(),
            method: method.to_uppercase(),
            status,
            latency_ms: (latency_ms * 10.0).round() / 10.0,
            cpu_active_time_s: (cpu_active * 10000.0).round() / 10000.0,
            energy_total_joules: (energy_joules * 1000000.0).round() / 1000000.0,
            co2e_grams: (co2e_grams * 1000000.0).round() / 1000000.0,
            cpu_utilization: (cpu_util * 100.0).round() / 100.0,
        };

        self.records.push(record.clone());
        record
    }
}

fn execute_http_request(url: &str, method: &str, body: Option<&str>, start_time: Instant) -> (u16, f64) {
    let clean = url.trim_start_matches("http://").trim_start_matches("https://");
    let parts: Vec<&str> = clean.splitn(2, '/').collect();
    let host_port = parts[0];
    let path = if parts.len() > 1 { format!("/{}", parts[1]) } else { "/".to_string() };

    let (host, port) = if host_port.contains(':') {
        let hp: Vec<&str> = host_port.split(':').collect();
        (hp[0], hp[1].parse::<u16>().unwrap_or(80))
    } else {
        (host_port, 80)
    };

    let addr_str = format!("{}:{}", host, port);
    let timeout = Duration::from_secs(5);

    if let Ok(mut stream) = TcpStream::connect_timeout(
        &addr_str.to_socket_addrs().ok().and_then(|mut a| a.next()).unwrap_or_else(|| "127.0.0.1:80".parse().unwrap()),
        timeout,
    ) {
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));

        let payload = body.unwrap_or("");
        let req_msg = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nAccept: application/json, text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            method.to_uppercase(), path, host, payload.len(), payload
        );

        let _ = stream.write_all(req_msg.as_bytes());
        let mut buffer = [0u8; 4096];
        let mut status = 200;

        if let Ok(bytes_read) = stream.read(&mut buffer) {
            status = parse_http_status(&buffer[..bytes_read]);
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        return (status, elapsed);
    }

    (500, start_time.elapsed().as_secs_f64())
}

fn parse_http_status(buffer: &[u8]) -> u16 {
    if buffer.is_empty() {
        return 200;
    }
    let resp_str = String::from_utf8_lossy(buffer);
    let Some(first_line) = resp_str.lines().next() else {
        return 200;
    };
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(st) = parts[1].parse::<u16>() {
            return st;
        }
    }
    200
}
