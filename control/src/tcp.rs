use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;

pub async fn wait_for_message(listener: &TcpListener) -> String {
    let (socket, _addr) = listener.accept().await.unwrap();
    let mut reader = BufReader::new(socket);

    let mut line = String::new();
    let mut content = String::new();
    let mut content_length = 0;

    loop {
        line.clear();
        reader.read_line(&mut line).await.unwrap();
        content.push_str(&line);

        if content_length == 0 {
            content_length = match line.strip_prefix("Content-Length: ") {
                Some(len) => len.trim().parse().unwrap(),
                None => 0,
            };
        }

        if line == "\r\n" {
            break;
        }
    }
    let mut buf = vec![0u8; content_length];
    reader.read_exact(&mut buf).await.unwrap();

    reader.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await.unwrap();

    std::str::from_utf8(&buf).unwrap().to_string()
}
