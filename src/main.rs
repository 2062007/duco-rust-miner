use anyhow::{Context, Result};
use hex::FromHex;
use rand::Rng;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::fs::File;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    task::JoinHandle,
    time::{sleep, Duration, Instant},
};

#[derive(Clone, Deserialize)]
struct Config {
    username: String,
    mining_key: String,
    difficulty: String,
    rig_identifier: String,
    thread_count: u32,
}

#[derive(Debug)]
struct Job {
    base: String,
    target: Vec<u8>,
    diff: u32,
}

#[derive(Debug)]
struct Solution {
    nonce: u32,
    elapsed_us: u128,
}

enum Feedback {
    Good,
    Bad(String),
    Block,
}

#[derive(Deserialize)]
struct PoolInfo {
    ip: String,
    port: u16,
}

async fn get_pool() -> Result<String> {
    let resp = reqwest::get("https://server.duinocoin.com/getPool")
        .await
        .context("GET pool")?
        .json::<PoolInfo>()
        .await
        .context("Parse pool JSON")?;
    Ok(format!("{}:{}", resp.ip, resp.port))
}

fn format_hashrate(hashrate: f32) -> String {
    if hashrate >= 1e9 {
        format!("{:.2} GH/s", hashrate / 1e9)
    } else if hashrate >= 1e6 {
        format!("{:.2} MH/s", hashrate / 1e6)
    } else if hashrate >= 1e3 {
        format!("{:.2} kH/s", hashrate / 1e3)
    } else {
        format!("{:.2} H/s", hashrate)
    }
}

fn solve(job: &Job) -> Option<Solution> {
    let sha_base = Sha1::new_with_prefix(job.base.as_bytes());
    let start = Instant::now();

    for nonce in 0..=(job.diff * 100) {
        let mut h = sha_base.clone();
        h.update(nonce.to_string());
        let hash = h.finalize();
        if hash.as_slice() == job.target {
            return Some(Solution {
                nonce,
                elapsed_us: start.elapsed().as_micros(),
            });
        }
    }
    None
}

async fn worker(cfg: Config, index: u32, multithread_id: u32) -> Result<()> {
    loop {
        let addr = match get_pool().await {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[worker{}] Pool error: {}", index, e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        println!("[worker{}] 🌐 Connect to {}", index, addr);
        let mut stream = match TcpStream::connect(&addr).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[worker{}] TCP connect error: {}", index, e);
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        let (reader, mut writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        // server version
        if reader.read_line(&mut line).await.is_ok() {
            println!("[worker{}] Connected (server v{})", index, line.trim());
        }
        line.clear();

        let mut accepted = 0;
        let mut rejected = 0;
        let mut t0 = Instant::now();

        loop {
            // Gửi request job
            let req = format!(
                "JOB,{},{},{}\n",
                cfg.username, cfg.difficulty, cfg.mining_key
            );
            if writer.write_all(req.as_bytes()).await.is_err() {
                break; // reconnect
            }

            // Nhận job
            if reader.read_line(&mut line).await.is_err() {
                break;
            }
            let jobline = line.trim().to_string();
            line.clear();
            let parts: Vec<&str> = jobline.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let base = parts[0].to_string();
            let target = Vec::from_hex(parts[1]).unwrap_or_default();
            let diff = parts[2].parse::<u32>().unwrap_or(0);
            let job = Job { base, target, diff };

            if let Some(sol) = solve(&job) {
                let hashrate = 1e6 * sol.nonce as f32 / sol.elapsed_us as f32;
                let msg = format!(
                    "{},{:.2},RustMiner,{},{}\n",
                    sol.nonce, hashrate, cfg.rig_identifier, multithread_id
                );
                if writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }

                if reader.read_line(&mut line).await.is_err() {
                    break;
                }
                let feedback = line.trim().to_string();
                line.clear();

                match feedback.as_str() {
                    "GOOD" => {
                        accepted += 1;
                        println!(
                            "[worker{}] ✅ Share accepted | {} | {}",
                            index,
                            format_hashrate(hashrate),
                            accepted
                        );
                    }
                    f if f.starts_with("BAD,") => {
                        rejected += 1;
                        println!(
                            "[worker{}] ❌ Rejected: {} (rej={})",
                            index,
                            &f[4..],
                            rejected
                        );
                    }
                    "BLOCK" => {
                        println!("[worker{}] ⛓️ New block", index);
                    }
                    _ => {
                        println!("[worker{}] ℹ️ {}", index, feedback);
                    }
                }

                if (accepted + rejected) % 10 == 0 {
                    let uptime = t0.elapsed().as_secs_f32();
                    println!(
                        "[worker{}] 📊 Shares: {} good / {} bad | Uptime {:.1}s",
                        index, accepted, rejected, uptime
                    );
                    t0 = Instant::now();
                }
            }
        }

        eprintln!("[worker{}] ⚠️ Lost connection, reconnecting...", index);
        sleep(Duration::from_secs(2)).await;
    }
}

async fn root() -> Result<()> {
    let file = File::open("config.yml").context("Open config.yml")?;
    let cfg: Config = serde_yaml::from_reader(file).context("Parse YAML")?;

    let multithread_id: u32 = rand::thread_rng().gen_range(10000..99999);

    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..cfg.thread_count {
        let cfg_clone = cfg.clone();
        let mtid = multithread_id;
        let h: JoinHandle<()> = tokio::spawn(async move {
            if let Err(e) = worker(cfg_clone, i, mtid).await {
                eprintln!("[worker{}] Error: {:?}", i, e);
            }
        });
        handles.push(h);
    }

    for h in handles {
        let _ = h.await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    root().await
}
