use std::fmt;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::{System, SystemExt, CpuExt, ProcessExt};
use chrono::Local;
use std::fs::OpenOptions;

// ================= STRUCTS =================

#[derive(Debug, Clone)]
struct CpuInfo {
    usage: f32,
}

#[derive(Debug, Clone)]
struct MemInfo {
    total: u64,
    used: u64,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: i32,
    name: String,
    cpu: f32,
}


#[derive(Debug, Clone)]
struct SystemSnapshot {
    cpu: CpuInfo,
    mem: MemInfo,
    processes: Vec<ProcessInfo>,
}

// ================= DISPLAY =================

impl fmt::Display for CpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPU Usage: {:.2}%", self.usage)
    }
}

impl fmt::Display for MemInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Memory: {} / {} MB", self.used, self.total)
    }
}

impl fmt::Display for ProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {} - {:.2}%", self.pid, self.name, self.cpu)
    }
}

impl fmt::Display for SystemSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.cpu)?;
        writeln!(f, "{}", self.mem)?;
        writeln!(f, "Top Processes:")?;
        for p in &self.processes {
            writeln!(f, "{}", p)?;
        }
        Ok(())
    }
}

// ================= SNAPSHOT =================

fn collect_snapshot() -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = CpuInfo {
        usage: sys.global_cpu_info().cpu_usage(),
    };

    let mem = MemInfo {
        total: sys.total_memory() / 1024,
        used: sys.used_memory() / 1024,
    };

    let mut processes: Vec<ProcessInfo> = sys.processes()
        .iter()
        .map(|(pid, p)| ProcessInfo {
            pid: pid.as_u32() as i32,
            name: p.name().to_string(),
            cpu: p.cpu_usage(),
        })
        .collect();

    processes.sort_by(|a, b| {
        b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal)
    });
    processes.truncate(5);

    SystemSnapshot { cpu, mem, processes }
}

// ================= FORMAT =================

fn format_response(snapshot: &SystemSnapshot, command: &str) -> String {
    match command.trim() {
        "cpu" => format!("{}", snapshot.cpu),
        "mem" => format!("{}", snapshot.mem),
        "ps" => {
            let mut out = String::from("Top Processes:\n");
            for p in &snapshot.processes {
                out.push_str(&format!("{}\n", p));
            }
            out
        }
        "all" => format!("{}", snapshot),
        "help" => String::from("Commands: cpu | mem | ps | all | help | quit"),
        "quit" => String::from("Bye!"),
        _ => String::from("Unknown command"),
    }
}

// ================= LOGGER =================

fn log_event(message: &str) {
    let now = Local::now();
    let log_line = format!("[{}] {}\n", now.format("%Y-%m-%d %H:%M:%S"), message);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("syswatch.log")
        .unwrap();

    file.write_all(log_line.as_bytes()).unwrap();
}

// ================= CLIENT =================

fn handle_client(mut stream: TcpStream, data: Arc<Mutex<SystemSnapshot>>) {
    log_event("Client connected");

    let mut buffer = [0; 1024];

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => return,
            Ok(n) => n,
            Err(_) => return,
        };

        let command = String::from_utf8_lossy(&buffer[..bytes_read]);
        log_event(&format!("Command: {}", command.trim()));

        let snapshot = data.lock().unwrap().clone();
        let response = format_response(&snapshot, &command);

        stream.write_all(response.as_bytes()).unwrap();

        if command.trim() == "quit" {
            break;
        }
    }
}

// ================= MAIN =================

fn main() {
    let snapshot = Arc::new(Mutex::new(collect_snapshot()));
    let data_clone = Arc::clone(&snapshot);

    thread::spawn(move || {
        loop {
            let mut data = data_clone.lock().unwrap();
            *data = collect_snapshot();
            thread::sleep(Duration::from_secs(5));
        }
    });

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    println!("Server running on port 7878...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let data = Arc::clone(&snapshot);
                thread::spawn(move || handle_client(stream, data));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
