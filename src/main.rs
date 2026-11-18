use std::env;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process;
use std::thread;
use std::time::Duration;

const IO_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REQUEST: usize = 16 * 1024;

fn handle_client(mut stream: TcpStream, pac_path: &str) {
    // Без таймаутов клиент может заблокировать обработку навсегда
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));

    let req = match read_request(&mut stream) {
        Ok(req) if !req.is_empty() => req,
        _ => return, // тут таймаут, ошибка сети или пустой запрос
    };

    let request = String::from_utf8_lossy(&req);
    let request_line = request.lines().next().unwrap_or("").trim();
    println!("[request size:{}] {request_line}", req.len());

    let result = match fs::read(pac_path) {
        Ok(content) => {
            let headers = format!(
                "HTTP/1.1 200 OK\r\n\
                 Content-Type: application/x-ns-proxy-autoconfig; charset=utf-8\r\n\
                 Content-Length: {}\r\n\
                 Cache-Control: no-cache, no-store, must-revalidate\r\n\
                 Pragma: no-cache\r\n\
                 Expires: 0\r\n\
                 Connection: close\r\n\r\n",
                content.len()
            );
            stream
                .write_all(headers.as_bytes())
                .and_then(|()| stream.write_all(&content))
        }
        Err(e) => {
            eprintln!("Ошибка чтения файла '{pac_path}': {e}");
            let (status, body) = if e.kind() == ErrorKind::NotFound {
                ("404 Not Found", "PAC file not found")
            } else {
                ("500 Internal Server Error", "PAC file read error")
            };
            let response = format!(
                "HTTP/1.1 {status}\r\n\
                 Content-Type: text/plain; charset=utf-8\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes())
        }
    };

    if let Err(e) = result {
        eprintln!("Ошибка отправки ответа: {e}");
    }
}

/// Читает запрос до конца заголовков (\r\n\r\n).
fn read_request(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 2048];
    loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.len() >= MAX_REQUEST || buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    Ok(buf)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} <host:port> <path/to/file.pac>", args[0]);
        process::exit(1);
    }

    let addr = &args[1];
    let pac_path = &args[2];

    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("Не удалось запустить сервер на {}: {}", addr, e);
        process::exit(1);
    });

    println!("PAC-сервер запущен на http://{addr}/ (файл отдаётся на любом пути)",);
    println!("Раздается файл: {}", pac_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let path = pac_path.clone();
                thread::spawn(move || handle_client(stream, &path));
            }
            Err(e) => eprintln!("Ошибка accept: {e}"),
        }
    }
}
