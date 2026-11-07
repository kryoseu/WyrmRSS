//! Small real HTTP fixture for conditional request and poller tests.
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};

pub async fn serve(responses: Vec<String>) -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/feed", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let mut requests = Vec::new();
        for response in responses {
            let (mut stream, _) =
                tokio::time::timeout(std::time::Duration::from_secs(10), listener.accept())
                    .await
                    .unwrap()
                    .unwrap();
            let mut request = Vec::new();
            loop {
                let mut chunk = [0; 1024];
                let n = stream.read(&mut chunk).await.unwrap();
                assert_ne!(n, 0);
                request.extend_from_slice(&chunk[..n]);
                if request.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
                assert!(request.len() < 32768);
            }
            requests.push(String::from_utf8(request).unwrap());
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.shutdown().await.unwrap();
        }
        requests
    });
    (url, task)
}

pub fn response(status: &str, headers: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: {}\r\n{headers}\r\n{body}",
        body.len()
    )
}
