use std::fs;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::path::Path;
use std::process;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Sender};
use std::thread;
extern crate dirs;
use std::io::Cursor;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn fetch_url(url: String, file_name: String) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(file_name)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}
fn start_listener(sender: Sender<String>, location: String) {
    let child = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .current_dir(location)
            .args(["/C", "sslocal", "--config", "config.json"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute process")
    } else {
        Command::new("bash")
            .current_dir(location)
            .arg("-c")
            .arg("sslocal")
            .arg("--config")
            .arg("config.json")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute process")
    };
    println!("Started process: {}", child.id());
    thread::spawn(move || {
        let mut f = BufReader::new(child.stdout.unwrap());
        loop {
            let mut buf = String::new();
            match f.read_line(&mut buf) {
                Ok(_) => {
                    sender.send(buf).unwrap();
                }
                Err(e) => println!("an error!: {:?}", e),
            }
        }
    });
}
#[tokio::main]
async fn main() -> std::io::Result<()> {
    const DATA: &[u8; 198] = b"{
    \"server\": \"gyattcentral.com\",
    \"server_port\": 8000,
    \"password\": \"Sa!9&diP!KZq7&e*X8h##DFfssn^y%\",
    \"method\": \"aes-256-gcm\",
    \"local_address\": \"127.0.0.1\",
    \"local_port\": 1080
}";

    let mut pos = 0;
    const FILENAME: &str = "config.json";
    let mut location = dirs::data_dir().expect("REASON");
    location.push("gyatt-dir");
    let cloned = location.clone();
    if Path::new(&location).exists() == false {
        fs::create_dir(&location)?
    }
    let mut config = location.clone();
    config.push(&FILENAME);
    if Path::new(&config).exists() == true {
        fs::remove_file(&config)?
    }

    let mut buffer = File::create(&config)?;

    while pos < DATA.len() {
        let bytes_written = buffer.write(&DATA[pos..])?;
        pos += bytes_written;
    }
    location.push("sslocal.exe");
    if Path::new(&location).exists() == false {
        let _ = fetch_url(
            "https://github.com/thatboyjake/personal/raw/main/sslocal.exe".to_string(),
            (&location.display()).to_string(),
        )
        .await;
    }
    if Path::new(&location).exists() == false {
        process::exit(1);
    }
    let (tx, rx) = channel();
    start_listener(tx, (cloned.display()).to_string());
    for line in rx {
        println!("Got this back: {}", line);
    }

    println!("Done!");
    Ok(())
}
