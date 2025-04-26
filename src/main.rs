use warp::Filter;
use tokio::net::UdpSocket;
use tokio::task;

#[tokio::main]
async fn main() {
    println!("VoiceBBS Server is starting...");

    // Spawn SIP server in a separate task
    task::spawn(async {
        run_sip_server().await;
    });

    // Warp HTTP server (already working)
    let hello = warp::path::end()
        .map(|| warp::reply::html("VoiceBBS is alive!"));

    warp::serve(hello)
        .run(([0, 0, 0, 0], 8080))
        .await;
}

async fn run_sip_server() {
    // Bind to UDP 5060
    let socket = UdpSocket::bind("0.0.0.0:5060")
        .await
        .expect("Failed to bind to UDP 5060");

    println!("SIP Server is listening on UDP 5060...");

    let mut buf = [0u8; 2048];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, addr)) => {
                let data = &buf[..size];
                let message = String::from_utf8_lossy(data);

                println!("Received from {}: {}", addr, message);

                if message.contains("INVITE") {
                    let response = build_basic_sip_response(&message);
                    let _ = socket.send_to(response.as_bytes(), &addr).await;
                    println!("Sent 200 OK to {}", addr);
                }
            }
            Err(e) => {
                eprintln!("UDP receive error: {}", e);
            }
        }
    }
}

fn build_basic_sip_response(invite: &str) -> String {
    // Very basic SIP 200 OK response
    let call_id = find_header(invite, "Call-ID").unwrap_or("random1234");
    let cseq = find_header(invite, "CSeq").unwrap_or("1 INVITE");
    let from = find_header(invite, "From").unwrap_or("<sip:unknown>");
    let to = find_header(invite, "To").unwrap_or("<sip:unknown>");

    format!(
        "SIP/2.0 200 OK\r\n\
         Via: SIP/2.0/UDP yourdomain.com;branch=z9hG4bK776asdhds\r\n\
         From: {}\r\n\
         To: {}\r\n\
         Call-ID: {}\r\n\
         CSeq: {}\r\n\
         Content-Length: 0\r\n\
         \r\n",
        from, to, call_id, cseq
    )
}

fn find_header<'a>(sip_message: &'a str, header: &str) -> Option<&'a str> {
    for line in sip_message.lines() {
        if line.starts_with(header) {
            return Some(line.trim());
        }
    }
    None
}