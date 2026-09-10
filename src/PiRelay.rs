use crate::helper::Helper::{CLI, Connection};
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const CHUNK_SIZE: usize = 1024 * 1024; // 1mb
const SMALL_FILE_SIZE: usize = 5 * 1024 * 1024; // 5 mb

pub struct PiRelayClient {
    client: Client,
    connection: Connection,
    username: String,
}

impl PiRelayClient {
    pub fn new(conn: Connection) -> Self {
        let uname = std::env::var("USER").unwrap_or("ANONYM".to_string());
        Self {
            client: Client::new(),
            connection: conn,
            username: uname,
        }
    }

    fn base_url(&self) -> String {
        format!("https://{}:{}", self.connection.ip, self.connection.port)
    }

    pub async fn send_config(&self, config_json: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/config", self.base_url());
        let response = self
            .client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(config_json.to_string())
            .send()
            .await?;
        if response.status().is_success() {
            println!("Config sent");
        } else {
            eprintln!("Failed to send config: {}", response.status());
        }
        Ok(())
    }

    pub async fn send_files(
        &self,
        send_map: &HashMap<String, Vec<PathBuf>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (resip, paths) in send_map {
            for p in paths {
                if p.is_dir() {
                    self.send_directory(resip, p, p).await?;
                } else {
                    let base = p.parent().unwrap_or(p);
                    self.send_file(resip, p, base).await?;
                }
            }
        }
        Ok(())
    }

    fn send_directory<'a>(
        &'a self,
        recipient: &'a str,
        dir_path: &'a Path,
        base_dir: &'a Path,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + 'a>,
    > {
        println!("Sending directory: {:?}", dir_path);
        Box::pin(async move {
            for entry in fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    self.send_file(recipient, &path, base_dir).await?;
                } else if path.is_dir() {
                    self.send_directory(recipient, &path, base_dir).await?;
                }
            }
            Ok(())
        })
    }

    async fn send_file(
        &self,
        recipient: &str,
        file_path: &Path,
        base_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rel_path = file_path
            .strip_prefix(base_dir)
            .unwrap_or(file_path)
            .to_str()
            .ok_or("Invalid path")?;
        let file_id = format!("{}_{}", self.username, rel_path.replace("/", "_"));
        let file_size = fs::metadata(file_path)?.len() as usize;
        println!(
            "Sending file: {:?} (Rel Path: {}, Size: {} B)",
            file_path, rel_path, file_size
        );
        if file_size <= SMALL_FILE_SIZE {
            self.send_small_file(recipient, &file_id, file_path, rel_path)
                .await?;
        } else {
            self.send_large_file_chunked(recipient, &file_id, file_path, rel_path)
                .await?;
        }

        Ok(())
    }

    async fn send_small_file(
        &self,
        recipient: &str,
        file_id: &str,
        file_path: &Path,
        relative_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/send/{}/{}", self.base_url(), recipient, file_id);
        let file_name = file_path
            .file_name()
            .ok_or("Invalid file name")?
            .to_str()
            .ok_or("Invalid file name")?;
        let file_content = fs::read(file_path)?;
        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(file_content).file_name(file_name.to_string()),
            )
            .text("file_id", file_id.to_string())
            .text("sender", self.username.clone())
            .text("relative_path", relative_path.to_string());
        let resp = self.client.post(&url).multipart(form).send().await?;
        if resp.status().is_success() {
            println!("Small file Sent: {}", relative_path);
        } else {
            eprintln!("Failed to Send Small file: {}", resp.status());
        }
        Ok(())
    }

    async fn send_large_file_chunked(
        &self,
        recipient: &str,
        file_id: &str,
        file_path: &Path,
        relative_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_data = fs::read(file_path)?;
        let tot_chunks = (file_data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
        println!("Sending large file in {} chunks", tot_chunks);
        for (chunk_idx, start) in (0..file_data.len()).step_by(CHUNK_SIZE).enumerate() {
            let end = (start + CHUNK_SIZE).min(file_data.len());
            let chunk = &file_data[start..end];
            let url = format!(
                "{}/send/{}/{}/{}",
                self.base_url(),
                recipient,
                file_id,
                chunk_idx
            );
            let resp = self
                .client
                .post(&url)
                .header("Content-Type", "application/octet-stream")
                .header("X-File-Id", file_id)
                .header("X-Chunk-Index", chunk_idx.to_string())
                .header("X-Total-Chunks", tot_chunks.to_string())
                .header("X-Sender", &self.username)
                .header("X-Relative-Path", relative_path)
                .body(chunk.to_vec())
                .send()
                .await?;

            if resp.status().is_success() {
                println!("Chunk {}/{} sent successfully", chunk_idx + 1, tot_chunks);
            } else {
                eprintln!(
                    "Failed to send chunk {}/{}: {}",
                    chunk_idx + 1,
                    tot_chunks,
                    resp.status()
                );
                return Err(format!("Failed to send chunk: {}", resp.status()).into());
            }
        }
        println!("Large file sent successfully");
        Ok(())
    }

    pub async fn receive_files(
        &self,
        senders: &[String],
        output_dir: Option<&PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if senders.is_empty() {
            self.receive_from_sender("", output_dir).await?;
        } else {
            for sender in senders {
                self.receive_from_sender(sender, output_dir).await?;
            }
        }
        Ok(())
    }

    async fn receive_from_sender(
        &self,
        sender: &str,
        output_dir: Option<&PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = if sender.is_empty() {
            format!("{}/receive/", self.base_url())
        } else {
            format!("{}/receive/{}", self.base_url(), sender)
        };
        println!(
            "Receiving files from: {}",
            if sender.is_empty() { "MailBox" } else { sender }
        );
        let resp = self
            .client
            .get(&url)
            .header("X-Receiver", &self.username)
            .send()
            .await?;
        if resp.status().is_success() {
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|ct| ct.to_str().ok())
                .unwrap_or("application/octet-stream");
            let file_name_header = resp
                .headers()
                .get("X-File-Name")
                .and_then(|f| f.to_str().ok())
                .map(|s| s.to_string());
            if content_type.contains("multipart") {
                self.handle_multipart_response(resp, output_dir).await?;
            } else {
                let data = resp.bytes().await?;
                if let Ok(files_info) = serde_json::from_slice::<Vec<FileInfo>>(&data) {
                    for file_info in files_info {
                        println!(
                            "Remote Unsynced: {} (size: {})",
                            file_info.name, file_info.size
                        );
                    }
                } else {
                    let file_name = file_name_header.unwrap_or_else(|| "received_file".to_string());
                    let output_path = if let Some(dir) = output_dir {
                        dir.join(file_name)
                    } else {
                        PathBuf::from(file_name)
                    };
                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&output_path, data)?;
                    println!("Received file saved to: {:?}", output_path);
                }
            }
        } else {
            eprintln!("Failed to receive files: {}", resp.status());
        }
        Ok(())
    }

    async fn handle_multipart_response(
        &self,
        response: reqwest::Response,
        output_dir: Option<&PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("application/octet-stream");
        let boundary = content_type
            .split("boundary=")
            .nth(1)
            .map(|b| b.trim().trim_matches('"'))
            .ok_or("No boundary found in multipart content-type")?;
        let boundary_marker = format!("--{}", boundary);

        let full_data = response.bytes().await?;
        let body_str = String::from_utf8_lossy(&full_data);

        let parts: Vec<&str> = body_str.split(&boundary_marker).collect();

        for part in parts.iter().skip(1) {
            if part.trim() == "--" || part.is_empty() {
                continue;
            }

            // Split headers from body
            let part_end = part.find("\r\n\r\n").unwrap_or(part.len());
            let headers = &part[..part_end];
            let body = &part[part_end + 4..];

            // Extract filename from Content-Disposition header
            let filename = headers
                .lines()
                .find(|line| line.to_lowercase().starts_with("content-disposition"))
                .and_then(|line| {
                    line.split("filename=")
                        .nth(1)
                        .map(|f| f.trim().trim_matches('"').to_string())
                })
                .unwrap_or_else(|| "unknown_file".to_string());

            // Clean up body (remove trailing \r\n if present)
            let body_data = body.trim_end_matches("\r\n").as_bytes();

            let output_path = if let Some(dir) = output_dir {
                dir.join(&filename)
            } else {
                PathBuf::from(&filename)
            };

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&output_path, body_data)?;
            println!("Received file saved to: {:?}", output_path);
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct FileInfo {
    name: String,
    size: u64,
}

#[derive(serde::Deserialize)]
struct ReceivedFile {
    name: String,
    data: Vec<u8>,
}

pub async fn send(clargs: &CLI) -> Result<(), Box<dyn std::error::Error>> {
    let client = PiRelayClient::new(clargs.connection.clone());
    client.send_files(&clargs.send).await?;
    Ok(())
}

pub async fn receive(clargs: &CLI) -> Result<(), Box<dyn std::error::Error>> {
    let client = PiRelayClient::new(clargs.connection.clone());
    client
        .receive_files(&clargs.receive, clargs.srcdir.as_ref())
        .await?;
    Ok(())
}

pub async fn send_config(
    clargs: &CLI,
    config_json: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = PiRelayClient::new(clargs.connection.clone());
    client.send_config(config_json).await?;
    Ok(())
}
