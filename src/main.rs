<<<<<<< HEAD
use std::fmt;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::{System, SystemExt, CpuExt, ProcessExt, PidExt};
use chrono::Local;
use std::fs::OpenOptions;

// ================= STRUCTS =================

#[derive(Debug, Clone)]
struct CpuInfo {
    usage: f32,
=======
// src/main.rs
use chrono::Local;
use std::fmt;
use sysinfo::{System, Process};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::fs::OpenOptions;

const AUTH_TOKEN: &str = "ENSPD2026";

// --- Types métier ---

#[derive(Debug, Clone)]
struct CpuInfo {
    usage_percent: f32,
    core_count: usize,
>>>>>>> db58036a330266de0b38ab9acd052cd9ba74ac57
}

#[derive(Debug, Clone)]
struct MemInfo {
<<<<<<< HEAD
    total: u64,
    used: u64,
=======
    total_mb: u64,
    used_mb: u64,
    free_mb: u64,
>>>>>>> db58036a330266de0b38ab9acd052cd9ba74ac57
}

#[derive(Debug, Clone)]
struct ProcessInfo {
<<<<<<< HEAD
    pid: i32,
    name: String,
    cpu: f32,
=======
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory_mb: u64,
>>>>>>> db58036a330266de0b38ab9acd052cd9ba74ac57
}

#[derive(Debug, Clone)]
struct SystemSnapshot {
<<<<<<< HEAD
    cpu: CpuInfo,
    mem: MemInfo,
    processes: Vec<ProcessInfo>,
}

// ================= DISPLAY =================

impl fmt::Display for CpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPU Usage: {:.2}%", self.usage)
=======
    timestamp: String,
    cpu: CpuInfo,
    memory: MemInfo,
    top_processes: Vec<ProcessInfo>,
}

// --- Affichage humain (Trait Display) ---

impl fmt::Display for CpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CPU: {:.1}% ({} cœurs)", self.usage_percent, self.core_count)
>>>>>>> db58036a330266de0b38ab9acd052cd9ba74ac57
    }
}

impl fmt::Display for MemInfo {
<<<<<<< HEAD
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Memory: {} / {} MB", self.used, self.total)
=======
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MEM: {}MB utilisés / {}MB total ({} MB libres)",
            self.used_mb, self.total_mb, self.free_mb
        )
>>>>>>> db58036a330266de0b38ab9acd052cd9ba74ac57
    }
}

impl fmt::Display for ProcessInfo {
<<<<<<< HEAD
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {} - {:.2}%", self.pid, self.name, self.cpu)
=======
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "  [{:>6}] {:<25} CPU:{:>5.1}%  MEM:{:>5}MB",
            self.pid, self.name, self.cpu_usage, self.memory_mb
        )
>>>>>>> db58036a330266de0b38ab9acd052cd9ba74ac57
    }
}

impl fmt::Display for SystemSnapshot {
<<<<<<< HEAD
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
=======
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== SysWatch — {} ===", self.timestamp)?;
        writeln!(f, "{}", self.cpu)?;
        writeln!(f, "{}", self.memory)?;
        writeln!(f, "--- Top Processus ---")?;
        for p in &self.top_processes {
            writeln!(f, "{}", p)?;
        }
        write!(f, "=====================")
    }
}

// --- Erreurs custom (exo 2) --- Etape 2: Gestion d'erreurs avec un enum dédié

#[derive(Debug)]
enum SysWatchError {
    CollectionFailed(String),
}

impl fmt::Display for SysWatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SysWatchError::CollectionFailed(msg) => write!(f, "Erreur collecte: {}", msg),
        }
    }
}

impl std::error::Error for SysWatchError {}

// --- Collecte système ---

fn collect_snapshot() -> Result<SystemSnapshot, SysWatchError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Petite pause pour que sysinfo ait des valeurs CPU non nulles
    std::thread::sleep(std::time::Duration::from_millis(500));
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let core_count = sys.cpus().len();

    if core_count == 0 {
        return Err(SysWatchError::CollectionFailed("Aucun CPU détecté".to_string()));
    }

    let total_mb = sys.total_memory() / 1024 / 1024;
    let used_mb = sys.used_memory() / 1024 / 1024;
    let free_mb = sys.free_memory() / 1024 / 1024;

    // Top 5 processus par consommation CPU
    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|p: &Process| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string(),
            cpu_usage: p.cpu_usage(),
            memory_mb: p.memory() / 1024 / 1024,
        })
        .collect();

    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
    processes.truncate(5);

    Ok(SystemSnapshot {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        cpu: CpuInfo { usage_percent: cpu_usage, core_count },
        memory: MemInfo { total_mb, used_mb, free_mb },
        top_processes: processes,
    })
}

//// Formatage responses (Exo 3) — Simuler une interface textuelle simple

fn format_response(snapshot: &SystemSnapshot, command: &str) -> String {
    let cmd = command.trim().to_lowercase();

    match cmd.as_str() {
        "cpu" => format!(
            "[CPU]\n{}\n\nHistorique:\n{}\n",
            snapshot.cpu,
            // Itérateur : simuler une barre de progression ASCII
            (0..10)
                .map(|i| {
                    let threshold = (snapshot.cpu.usage_percent / 10.0) as usize;
                    if i < threshold { "█" } else { "░" }
                })
                .collect::<Vec<_>>()
                .join("") + &format!(" {:.1}%", snapshot.cpu.usage_percent)
        ),

        "mem" => {
            let percent = (snapshot.memory.used_mb as f64 / snapshot.memory.total_mb as f64) * 100.0;
            let bar: String = (0..20)
                .map(|i| if i < (percent / 5.0) as usize { '█' } else { '░' })
                .collect();
            format!(
                "[MÉMOIRE]\n{}\n[{}] {:.1}%\n",
                snapshot.memory, bar, percent
            )
        },

        "ps" | "procs" => {
            let lines: String = snapshot
                .top_processes
                .iter()
                .enumerate()
                .map(|(i, p)| format!("{}. {}", i + 1, p))
                .collect::<Vec<_>>()
                .join("\n");
            format!("[PROCESSUS — Top {}]\n{}\n", snapshot.top_processes.len(), lines)
        },

        "shutdown" => {
            // Windows
            std::process::Command::new("shutdown")
                .args(["/s", "/t", "5"])
                .spawn()
                .ok();
            "SHUTDOWN programmé dans 5 secondes.\n".to_string()
        }

        "reboot" => {
            std::process::Command::new("shutdown")
                .args(["/r", "/t", "5"])
                .spawn()
                .ok();
            "REBOOT programmé dans 5 secondes.\n".to_string()
        }

        "abort" => {
            // Annuler un shutdown/reboot en cours
            std::process::Command::new("shutdown")
                .args(["/a"])
                .spawn()
                .ok();
            "Extinction annulée.\n".to_string()
        }

        _ if cmd.starts_with("msg ") => {
            // Afficher un message dans le terminal de l'étudiant
            // msg Bonjour tout le monde !
            let text = &cmd[4..];
            println!("\n╔══════════════════════════════════════╗");
            println!("║  MESSAGE DU PROFESSEUR               ║");
            println!("║  {}{}║", text, " ".repeat(38usize.saturating_sub(text.len())));
            println!("╚══════════════════════════════════════╝\n");
            format!("Message affiché sur la machine cible.\n")
        }

        _ if cmd.starts_with("install ") => {
            // install <nom-du-package-winget>
            // ex: install git.git
            let package = cmd[8..].trim().to_string();
            std::thread::spawn(move || {
                std::process::Command::new("winget")
                    .args(["install", "--silent", &package])
                    .status()
                    .ok();
            });
            format!("Installation de '{}' lancée en arrière-plan.\n", &cmd[8..])
        }

        "all" | "" => format!("{}\n", snapshot),

        "help" => concat!(
            "Commandes disponibles:\n",
            "  cpu   — Usage CPU + barre\n",
            "  mem   — Mémoire RAM\n",
            "  ps    — Top processus\n",
            "  all   — Vue complète\n",
            "  help  — Cette aide\n",
            "  quit  — Fermer la connexion\n",
        ).to_string(),

        "quit" | "exit" => "BYE\n".to_string(),

        _ => format!("Commande inconnue: '{}'. Tape 'help'.\n", command.trim()),
    }
}


// // Exo 4: Serveur TCP multithreadé —
// fn handle_client(mut stream: TcpStream, snapshot: Arc<Mutex<SystemSnapshot>>) {
//     let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or("inconnu".to_string());
//     println!("[+] Connexion de {}", peer);
//     log_event(&format!("[+] Connexion de {}", peer));

//     // Message de bienvenue
//     let welcome = concat!(
//         "╔══════════════════════════════╗\n",
//         "║   SysWatch v1.0 — ENSPD      ║\n",
//         "║   Tape 'help' pour commencer ║\n",
//         "╚══════════════════════════════╝\n",
//         "> "
//     );
//     let _ = stream.write_all(welcome.as_bytes());

//     let reader = BufReader::new(stream.try_clone().expect("Clone stream échoué"));

//     for line in reader.lines() {
//         match line {
//             Ok(cmd) => {
//                 let cmd = cmd.trim().to_string();
//                 println!("[{}] commande: '{}'", peer, cmd);
//                 log_event(&format!("[{}] commande: '{}'", peer, cmd));

//                 if cmd.eq_ignore_ascii_case("quit") || cmd.eq_ignore_ascii_case("exit") {
//                     let _ = stream.write_all(b"Au revoir!\n");
//                     break;
//                 }

//                 // Lire le snapshot partagé (thread-safe)
//                 let response = {
//                     let snap = snapshot.lock().unwrap();
//                     format_response(&snap, &cmd)
//                 };

//                 let _ = stream.write_all(response.as_bytes());
//                 let _ = stream.write_all(b"> "); // prompt
//             }
//             Err(_) => break,
//         }
//     }

//     println!("[-] Déconnexion de {}", peer);
//     log_event(&format!("[-] Déconnexion de {}", peer));
// }

fn snapshot_refresher(snapshot: Arc<Mutex<SystemSnapshot>>) {
    loop {
        thread::sleep(Duration::from_secs(5));
        match collect_snapshot() {
            Ok(new_snap) => {
                let mut snap = snapshot.lock().unwrap();
                *snap = new_snap;
                println!("[refresh] Métriques mises à jour");
            }
            Err(e) => eprintln!("[refresh] Erreur: {}", e),
        }
    }
}


fn log_event(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let line = format!("[{}] {}\n", timestamp, message);

    // Écriture console
    print!("{}", line);

    // Écriture fichier — on ignore l'erreur silencieusement (best-effort)
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("syswatch.log")
    {
        let _ = file.write_all(line.as_bytes());
    }
}


fn handle_client(mut stream: TcpStream, snapshot: Arc<Mutex<SystemSnapshot>>) {
    let peer = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or("inconnu".to_string());
    log_event(&format!("[+] Connexion de {}", peer));

    // Étape 1 : demander le token
    let _ = stream.write_all(b"TOKEN: ");
    let mut reader = BufReader::new(stream.try_clone().expect("Clone failed"));
    let mut token_line = String::new();
    if reader.read_line(&mut token_line).is_err() || token_line.trim() != AUTH_TOKEN {
        let _ = stream.write_all(b"UNAUTHORIZED\n");
        log_event(&format!("[!] Accès refusé depuis {}", peer));
        return;
    }
    let _ = stream.write_all(b"OK\n");
    log_event(&format!("[✓] Authentifié: {}", peer));

    // Boucle de commandes
    for line in reader.lines() {
        match line {
            Ok(cmd) => {
                let cmd = cmd.trim().to_string();
                log_event(&format!("[{}] commande: '{}'", peer, cmd));

                if cmd.eq_ignore_ascii_case("quit") {
                    let _ = stream.write_all(b"BYE\n");
                    break;
                }

                let response = {
                    let snap = snapshot.lock().unwrap();
                    format_response(&snap, &cmd)
                };

                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(b"\nEND\n"); // marqueur fin de réponse
            }
            Err(_) => break,
        }
    }

    log_event(&format!("[-] Déconnexion de {}", peer));
}


// Main Exo 1: Types métier et affichage — Etape 3: Affichage humain avec le trait Display
// fn main() {
//     // Test d'affichage — données fictives pour valider les types
//     let snapshot = SystemSnapshot {
//         timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
//         cpu: CpuInfo { usage_percent: 42.5, core_count: 8 },
//         memory: MemInfo { total_mb: 16384, used_mb: 8192, free_mb: 8192 },
//         top_processes: vec![
//             ProcessInfo { pid: 1234, name: "code.exe".to_string(), cpu_usage: 12.3, memory_mb: 512 },
//             ProcessInfo { pid: 5678, name: "chrome.exe".to_string(), cpu_usage: 8.1, memory_mb: 1024 },
//         ],
//     };

//     println!("{}", snapshot);
// }


// Main Exo 2: Gestion d'erreurs — Etape 1: Utilisation de Result dans la fonction de collecte et affichage complet

// fn main() {
//     match collect_snapshot() {
//         Ok(snapshot) => println!("{}", snapshot),
//         Err(e) => eprintln!("ERREUR: {}", e),
//     }
// }


// Main Exo 3: Formatage de réponses — Simuler une interface textuelle simple

// fn main() {
//     let snapshot = collect_snapshot().expect("Collecte échouée");
//     println!("{}", format_response(&snapshot, "cpu"));
//     println!("{}", format_response(&snapshot, "mem"));
//     println!("{}", format_response(&snapshot, "ps"));
//     println!("{}", format_response(&snapshot, "help"));
// }

// Main Exo 4: Serveur TCP multithreadé — Etape 1: Lancement d'un serveur TCP basique


fn main() {
    println!("SysWatch démarrage...");

    // Collecte initiale
    let initial = collect_snapshot().expect("Impossible de collecter les métriques initiales");
    println!("Métriques initiales OK:\n{}", initial);

    // Snapshot partagé entre tous les threads
    let shared_snapshot = Arc::new(Mutex::new(initial));

    // Thread de rafraîchissement automatique toutes les 5s
    {
        let snap_clone = Arc::clone(&shared_snapshot);
        thread::spawn(move || snapshot_refresher(snap_clone));
    }

    // Démarrage du serveur TCP
    let listener = TcpListener::bind("0.0.0.0:7878").expect("Impossible de bind le port 7878");
    println!("Serveur en écoute sur port 7878...");
    println!("Connecte-toi avec: telnet localhost 7878");
    println!("  ou: nc localhost 7878 (WSL/Git Bash)");
    println!("  ou: Test-NetConnection localhost -Port 7878 (PowerShell - test seulement)");
    println!("Ctrl+C pour arrêter.\n");
>>>>>>> db58036a330266de0b38ab9acd052cd9ba74ac57

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
<<<<<<< HEAD
                let data = Arc::clone(&snapshot);
                thread::spawn(move || handle_client(stream, data));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
=======
                let snap_clone = Arc::clone(&shared_snapshot);
                thread::spawn(move || handle_client(stream, snap_clone));
            }
            Err(e) => eprintln!("Erreur connexion entrante: {}", e),
        }
    }
}
>>>>>>> db58036a330266de0b38ab9acd052cd9ba74ac57
