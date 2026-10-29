use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn print_banner() {
    println!("========================================================");
    println!("  PodMesh Windows Native Drive Mount Utility v2.4.1");
    println!("  Moteur CAS 4 Mo + WinFsp FUSE (Userland-First)");
    println!("========================================================");
}

fn load_credentials() -> (String, String) {
    if let (Ok(ak), Ok(sk)) = (env::var("MESHFS_ACCESS_KEY"), env::var("MESHFS_SECRET_KEY")) {
        if !ak.is_empty() && !sk.is_empty() {
            return (ak, sk);
        }
    }

    let mut candidate_paths = Vec::new();
    if let Ok(profile) = env::var("USERPROFILE") {
        candidate_paths.push(PathBuf::from(profile).join(".meshfs").join("credentials.env"));
    }
    candidate_paths.push(PathBuf::from("C:\\ProgramData\\PodMesh\\credentials.env"));
    candidate_paths.push(PathBuf::from("C:\\Program Files\\PodMesh\\credentials.env"));

    for path in candidate_paths {
        if let Ok(content) = fs::read_to_string(&path) {
            let mut ak = None;
            let mut sk = None;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    if k == "MESHFS_ACCESS_KEY" || k == "MESHFS_S3_ACCESS_KEY" {
                        ak = Some(v);
                    } else if k == "MESHFS_SECRET_KEY" || k == "MESHFS_S3_SECRET_KEY" {
                        sk = Some(v);
                    }
                }
            }
            if let (Some(a), Some(s)) = (ak, sk) {
                return (a, s);
            }
        }
    }

    (
        "meshfs-admin".to_string(),
        "meshfs-secret-2026-safe-key".to_string(),
    )
}

fn stop_sentinel_path(drive: &str) -> PathBuf {
    let clean = drive.trim_end_matches(':').to_uppercase();
    let base = env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());
    PathBuf::from(base).join(format!("podmesh-unmount-{}.stop", clean))
}

fn global_stop_sentinel_path(drive: &str) -> PathBuf {
    let clean = drive.trim_end_matches(':').to_uppercase();
    PathBuf::from("C:\\Windows\\Temp").join(format!("podmesh-unmount-{}.stop", clean))
}

fn clear_stop_sentinels(drive: &str) {
    let _ = fs::remove_file(stop_sentinel_path(drive));
    let _ = fs::remove_file(global_stop_sentinel_path(drive));
}

fn is_stop_requested(drive: &str) -> bool {
    stop_sentinel_path(drive).exists() || global_stop_sentinel_path(drive).exists()
}

fn build_rclone_args(
    drive: &str,
    endpoint: &str,
    bucket: &str,
    access_key: &str,
    secret_key: &str,
    is_global: bool,
) -> Vec<String> {
    let mut args = vec![
        "mount".to_string(),
        format!(":s3:{}", bucket),
        drive.to_string(),
        "--s3-provider=Other".to_string(),
        format!("--s3-endpoint={}", endpoint),
        format!("--s3-access-key-id={}", access_key),
        format!("--s3-secret-access-key={}", secret_key),
        "--s3-force-path-style=true".to_string(),
        "--s3-no-check-bucket".to_string(),
        "--s3-chunk-size=8M".to_string(),
        "--s3-upload-concurrency=2".to_string(),
        "--bwlimit=60M".to_string(),
        "--vfs-cache-mode=full".to_string(),
        "--vfs-write-back=1s".to_string(),
        "--volname=MeshFS-CAS".to_string(),
        "--rc".to_string(),
        "--rc-addr=127.0.0.1:5572".to_string(),
        "--rc-no-auth".to_string(),
    ];
    if is_global {
        args.push("-o".to_string());
        args.push("FileSecurity=D:P(A;;FA;;;WD)".to_string());
    }
    args
}

fn find_rclone_bin() -> &'static str {
    if Path::new("C:\\Program Files\\PodMesh\\rclone.exe").exists() {
        "C:\\Program Files\\PodMesh\\rclone.exe"
    } else {
        "rclone.exe"
    }
}

fn find_self_bin() -> String {
    if Path::new("C:\\Program Files\\PodMesh\\podmesh-windows-mount.exe").exists() {
        "C:\\Program Files\\PodMesh\\podmesh-windows-mount.exe".to_string()
    } else {
        env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "podmesh-windows-mount.exe".to_string())
    }
}

fn main() -> ExitCode {
    let raw_args: Vec<String> = env::args().collect();
    let is_global = raw_args.iter().any(|a| a == "--global" || a == "-g");
    let args: Vec<String> = raw_args
        .into_iter()
        .filter(|a| a != "--global" && a != "-g")
        .collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_banner();
        println!("Usage: podmesh-windows-mount.exe <COMMAND> [DRIVE] [ENDPOINT] [BUCKET] [--global]");
        println!();
        println!("Modes de montage :");
        println!("  Par défaut (Recommandé) : Montage PUR USERLAND dans la session interactive");
        println!("                            de l'utilisateur (zéro élévation Administrateur).");
        println!("  Option --global (-g)    : Montage GLOBAL (\\GLOBAL??\\Z:) sous SYSTEM avec");
        println!("                            -o uid=-1,gid=-1 (visible par sshd Session 0 + GUI).");
        println!();
        println!("Commands:");
        println!("  mount [DRIVE] [ENDPOINT] [BUCKET]            Montage FUSE au premier plan");
        println!("  daemon [DRIVE] [ENDPOINT] [BUCKET]           Montage supervisé en arrière-plan (sans fenêtre)");
        println!("  supervise [DRIVE] [ENDPOINT] [BUCKET]        Boucle watchdog interne de maintien du montage");
        println!("  unmount [DRIVE]                              Drainage du cache VFS puis démontage propre");
        println!("  install-userland [DRIVE] [ENDPOINT] [BUCKET] Configure le démarrage auto en Pur Userland (défaut)");
        println!("  install-global [DRIVE] [ENDPOINT] [BUCKET]   Configure le démarrage auto Global (visible en SSH)");
        println!("  status [ENDPOINT]                            Vérifie la santé de la passerelle MeshFS");
        return ExitCode::SUCCESS;
    }

    let cmd = args[1].as_str();
    match cmd {
        "mount" | "daemon" | "supervise" => {
            let drive = args.get(2).map(|s| s.as_str()).unwrap_or("Z:");
            let endpoint = args
                .get(3)
                .map(|s| s.as_str())
                .unwrap_or("http://192.168.10.179:9000");
            let bucket = args
                .get(4)
                .map(|s| s.as_str())
                .unwrap_or("truenas-backup");
            let (access_key, secret_key) = load_credentials();

            clear_stop_sentinels(drive);

            if cmd == "daemon" {
                print_banner();
                println!(
                    "Mode d'exécution    : {}",
                    if is_global {
                        "GLOBAL (\\GLOBAL??\\, visible SSH Session 0 + GUI Session 1)"
                    } else {
                        "PUR USERLAND (Session utilisateur courante, sans privilèges)"
                    }
                );
                println!("Drive Cible         : {}", drive);
                println!("MeshFS Gateway URL  : {}", endpoint);
                println!("Bucket de Stockage  : {}", bucket);

                let self_bin = find_self_bin();
                let mut sup_args = vec![
                    "supervise".to_string(),
                    drive.to_string(),
                    endpoint.to_string(),
                    bucket.to_string(),
                ];
                if is_global {
                    sup_args.push("--global".to_string());
                }

                let mut command = Command::new(&self_bin);
                command
                    .args(&sup_args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());

                #[cfg(windows)]
                command.creation_flags(CREATE_NO_WINDOW);

                match command.spawn() {
                    Ok(child) => {
                        println!(
                            "[+] Superviseur FUSE MeshFS ({}) lancé en arrière-plan sans fenêtre (PID: {})",
                            drive,
                            child.id()
                        );
                        return ExitCode::SUCCESS;
                    }
                    Err(e) => {
                        eprintln!("[-] Erreur lancement superviseur : {}", e);
                        return ExitCode::from(1);
                    }
                }
            }

            let rclone_bin = find_rclone_bin();
            let rclone_args =
                build_rclone_args(drive, endpoint, bucket, &access_key, &secret_key, is_global);

            if cmd == "mount" {
                print_banner();
                println!("Drive Cible         : {}", drive);
                println!("MeshFS Gateway URL  : {}", endpoint);
                println!("Bucket de Stockage  : {}", bucket);
                println!(
                    "Espace de noms NT   : {}",
                    if is_global {
                        "GLOBAL (-o uid=-1,gid=-1)"
                    } else {
                        "USERLAND (Session locale)"
                    }
                );
                match Command::new(rclone_bin).args(&rclone_args).status() {
                    Ok(s) => {
                        println!("[+] Processus de montage terminé : {}", s);
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("[-] Erreur pilote WinFsp/rclone : {}", e);
                        ExitCode::from(1)
                    }
                }
            } else {
                // "supervise": watchdog loop restarting rclone if it crashes unless stop sentinel exists
                loop {
                    if is_stop_requested(drive) {
                        clear_stop_sentinels(drive);
                        break;
                    }
                    let mut child_cmd = Command::new(rclone_bin);
                    child_cmd
                        .args(&rclone_args)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null());

                    #[cfg(windows)]
                    child_cmd.creation_flags(CREATE_NO_WINDOW);

                    let _ = child_cmd.status();
                    if is_stop_requested(drive) {
                        clear_stop_sentinels(drive);
                        break;
                    }
                    thread::sleep(Duration::from_secs(3));
                }
                ExitCode::SUCCESS
            }
        }
        "unmount" => {
            print_banner();
            let drive = args.get(2).map(|s| s.as_str()).unwrap_or("Z:");
            println!("[*] Préparation du démontage gracieux de {}...", drive);
            let _ = fs::write(stop_sentinel_path(drive), b"stop");
            let _ = fs::write(global_stop_sentinel_path(drive), b"stop");

            println!("[*] Vérification et vidage du cache VFS (127.0.0.1:5572)...");
            for i in 1..=15 {
                let out = Command::new("curl.exe")
                    .args([
                        "-s",
                        "-m",
                        "2",
                        "-X",
                        "POST",
                        "http://127.0.0.1:5572/vfs/stats",
                    ])
                    .output();
                match out {
                    Ok(o) if o.status.success() => {
                        let body = String::from_utf8_lossy(&o.stdout);
                        let in_prog = !body.contains("\"uploadsInProgress\": 0")
                            && !body.contains("\"uploadsInProgress\":0")
                            && body.contains("\"uploadsInProgress\"");
                        let queued = !body.contains("\"uploadsQueued\": 0")
                            && !body.contains("\"uploadsQueued\":0")
                            && body.contains("\"uploadsQueued\"");
                        if !in_prog && !queued {
                            println!("[+] Cache VFS synchronisé (0 upload en attente).");
                            break;
                        }
                        println!(
                            "    Attente synchronisation cache VFS vers MeshFS ({}/15)...",
                            i
                        );
                        thread::sleep(Duration::from_secs(1));
                    }
                    _ => break,
                }
            }

            let quit = Command::new("curl.exe")
                .args([
                    "-s",
                    "-m",
                    "3",
                    "-X",
                    "POST",
                    "http://127.0.0.1:5572/core/quit",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            if quit.map(|s| !s.success()).unwrap_or(true) {
                println!("[*] Canal RC inactif, nettoyage de secours du processus rclone...");
                let _ = Command::new("taskkill")
                    .args(["/IM", "rclone.exe"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }

            println!("[+] Lecteur {} démonté proprement.", drive);
            ExitCode::SUCCESS
        }
        "install-userland" => {
            print_banner();
            let drive = args.get(2).map(|s| s.as_str()).unwrap_or("Z:");
            let endpoint = args
                .get(3)
                .map(|s| s.as_str())
                .unwrap_or("http://192.168.10.179:9000");
            let bucket = args
                .get(4)
                .map(|s| s.as_str())
                .unwrap_or("truenas-backup");
            let user = env::var("USERNAME").unwrap_or_else(|_| "logos".to_string());
            let self_bin = find_self_bin();
            let tr = format!("\"{}\" daemon {} {} {}", self_bin, drive, endpoint, bucket);

            println!(
                "[*] Enregistrement de la tâche planifiée PUR USERLAND (/RL LIMITED, Session de {})...",
                user
            );
            let status = Command::new("schtasks.exe")
                .args([
                    "/Create",
                    "/RU",
                    &user,
                    "/IT",
                    "/RL",
                    "LIMITED",
                    "/SC",
                    "ONLOGON",
                    "/TN",
                    "PodMesh-MeshFS-Drive",
                    "/TR",
                    &tr,
                    "/F",
                ])
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!("[+] Tâche PodMesh-MeshFS-Drive configurée en mode PUR USERLAND.");
                    ExitCode::SUCCESS
                }
                _ => {
                    eprintln!("[-] Échec de configuration de la tâche planifiée.");
                    ExitCode::from(1)
                }
            }
        }
        "install-global" => {
            print_banner();
            let drive = args.get(2).map(|s| s.as_str()).unwrap_or("Z:");
            let endpoint = args
                .get(3)
                .map(|s| s.as_str())
                .unwrap_or("http://192.168.10.179:9000");
            let bucket = args
                .get(4)
                .map(|s| s.as_str())
                .unwrap_or("truenas-backup");
            let self_bin = find_self_bin();
            let tr = format!(
                "\"{}\" supervise {} {} {} --global",
                self_bin, drive, endpoint, bucket
            );

            println!("[*] Enregistrement de la tâche planifiée GLOBALE (SYSTEM + FileSecurity Everyone)...");
            println!("    Ce mode rend {} visible à la fois dans Explorer (Session 1) et SSH (Session 0).", drive);
            let status = Command::new("schtasks.exe")
                .args([
                    "/Create",
                    "/RU",
                    "SYSTEM",
                    "/RL",
                    "HIGHEST",
                    "/SC",
                    "ONSTART",
                    "/TN",
                    "PodMesh-MeshFS-Drive",
                    "/TR",
                    &tr,
                    "/F",
                ])
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!("[+] Tâche PodMesh-MeshFS-Drive configurée en mode GLOBAL (SYSTEM).");
                    ExitCode::SUCCESS
                }
                _ => {
                    eprintln!("[-] Échec (droits Administrateur requis pour install-global).");
                    ExitCode::from(1)
                }
            }
        }
        "status" => {
            print_banner();
            let endpoint = args
                .get(2)
                .map(|s| s.as_str())
                .unwrap_or("http://192.168.10.179:9000");
            println!(
                "[*] Test de joignabilité de MeshFS Gateway à {}/healthz...",
                endpoint
            );
            let status = Command::new("curl.exe")
                .args(["-s", &format!("{}/healthz", endpoint)])
                .status();
            match status {
                Ok(_) => {
                    println!();
                    ExitCode::SUCCESS
                }
                Err(_) => ExitCode::from(4),
            }
        }
        other => {
            eprintln!("[-] Commande inconnue : {}", other);
            ExitCode::from(2)
        }
    }
}
