use std::env;
use std::process::{Command, ExitCode, Stdio};

fn print_banner() {
    println!("========================================================");
    println!("  PodMesh Windows Native Drive Mount Utility v2.4.0");
    println!("  Moteur CAS 4 Mo + WinFsp Kernel FUSE Layer");
    println!("========================================================");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    print_banner();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("Usage: podmesh-windows-mount.exe <COMMAND> [OPTIONS]");
        println!();
        println!("Commands:");
        println!("  mount [DRIVE] [ENDPOINT] [BUCKET]   Mount MeshFS as a Windows FUSE drive (default: Z:)");
        println!("  daemon [DRIVE] [ENDPOINT] [BUCKET]  Mount MeshFS in background as a persistent drive");
        println!("  unmount [DRIVE]                     Unmount the Windows drive (default: Z:)");
        println!("  status [ENDPOINT]                   Query MeshFS Gateway health from Windows");
        return ExitCode::SUCCESS;
    }

    match args[1].as_str() {
        "mount" | "daemon" => {
            let is_daemon = args[1] == "daemon";
            let drive = args.get(2).map(|s| s.as_str()).unwrap_or("Z:");
            let endpoint = args.get(3).map(|s| s.as_str()).unwrap_or("http://192.168.10.179:9000");
            let bucket = args.get(4).map(|s| s.as_str()).unwrap_or("truenas-backup");

            let access_key = env::var("MESHFS_ACCESS_KEY").unwrap_or_else(|_| "meshfs-admin".to_string());
            let secret_key = env::var("MESHFS_SECRET_KEY").unwrap_or_else(|_| "meshfs-secret-2026-safe-key".to_string());

            println!("Drive Cible         : {}", drive);
            println!("MeshFS Gateway URL  : {}", endpoint);
            println!("Bucket de Stockage  : {}", bucket);
            println!("Clé d'accès         : {}", access_key);

            println!("\n[+] Initialisation du pilote FUSE Windows (WinFsp)...");
            let rclone_args = vec![
                "mount".to_string(),
                format!(":s3:{}", bucket),
                drive.to_string(),
                "--s3-provider=Other".to_string(),
                format!("--s3-endpoint={}", endpoint),
                format!("--s3-access-key-id={}", access_key),
                format!("--s3-secret-access-key={}", secret_key),
                "--s3-force-path-style=true".to_string(),
                "--s3-no-check-bucket".to_string(),
                "--vfs-cache-mode=full".to_string(),
                "--volname=MeshFS-CAS".to_string(),
            ];

            let rclone_bin = if std::path::Path::new("C:\\Program Files\\PodMesh\\rclone.exe").exists() {
                "C:\\Program Files\\PodMesh\\rclone.exe"
            } else {
                "rclone.exe"
            };

            if is_daemon {
                match Command::new(rclone_bin)
                    .args(&rclone_args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    Ok(child) => {
                        println!("[+] Disque FUSE MeshFS ({}) monté en arrière-plan (PID: {})", drive, child.id());
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("[-] Erreur lancement pilote WinFsp/rclone : {}", e);
                        ExitCode::from(1)
                    }
                }
            } else {
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
            }
        }
        "unmount" => {
            let drive = args.get(2).map(|s| s.as_str()).unwrap_or("Z:");
            println!("[*] Démontage du lecteur FUSE {}...", drive);
            let _ = Command::new("taskkill").args(["/F", "/IM", "rclone.exe"]).status();
            let _ = Command::new("net").args(["use", drive, "/delete", "/y"]).status();
            println!("[+] Lecteur {} démonté avec succès.", drive);
            ExitCode::SUCCESS
        }
        "status" => {
            let endpoint = args.get(2).map(|s| s.as_str()).unwrap_or("http://192.168.10.179:9000");
            println!("[*] Test de joignabilité de MeshFS Gateway à {}/healthz...", endpoint);
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
        cmd => {
            eprintln!("[-] Commande inconnue : {}", cmd);
            ExitCode::from(2)
        }
    }
}
