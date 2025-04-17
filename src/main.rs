use std::{fs::File, sync::Arc};

use serde_derive::Deserialize;
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpListener, TcpStream}, spawn};

#[derive(Deserialize)]
struct AppConfig {
    pub bind :String,
    pub minecraft :String,
    pub webserver :String
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let file = File::open("./config.yml")?;
    let app_config :AppConfig = serde_yaml::from_reader(file).unwrap();
    let app_config = Arc::new(app_config);
    let listener = TcpListener::bind(&app_config.bind).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let config = Arc::clone(&app_config);
        spawn(async move { 
            match handle_client(stream, config).await {
                Ok(_) => {},
                Err(e) => panic!("{:?}", e),
            }
        });
    }
}

async fn handle_client(mut _stream :TcpStream, config :Arc<AppConfig>) -> io::Result<()> {
    let mut buf = [0; 1024];
    let mut http = false;
    let (mut cread, mut cwrite) = _stream.into_split();

    let n = cread.read(&mut buf).await?;
    let b = split_byte_array(&buf[0..n], 32);
    if !b.is_empty() {
        let s  :String =b.first().iter().fold(String::new(), |x ,y| x + String::from_utf8(y.to_vec()).unwrap_or("".to_string()).as_str());
        if ["HEAD", "GET", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE"].contains(&s.as_str()) {
           http = true; 
        }
    }
    
    let server_addr = if http  { config.webserver.as_str() } else { config.minecraft.as_str() };
    let server = TcpStream::connect(server_addr).await?;

    let (mut sread, mut swrite) = server.into_split(); 

    spawn(async move { proxy(&mut cread, &mut swrite, Some(&buf[0..n])).await });
    spawn(async move { proxy(&mut sread, &mut cwrite, None).await });
    Ok(())
}

async fn proxy(from :&mut OwnedReadHalf, to :&mut OwnedWriteHalf, first_u8 :Option<&[u8]>) {
    let mut first = first_u8;
    
    if let Some(f) = first.take() {
        let _ = to.write(f).await;
    }

    let _ = io::copy(from, to).await;
}

fn split_byte_array(data: &[u8], delimiter: u8) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut current_part = Vec::new();

    for &byte in data {
        if byte == delimiter {
            result.push(current_part);
            current_part = Vec::new();
        } else {
            current_part.push(byte);
        }
    }

    result.push(current_part);
    result
}