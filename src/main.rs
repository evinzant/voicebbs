use std::net::UdpSocket;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:5060")?;
    println!("VoiceBBS SIP server listening on UDP 5060");

    loop {
        let mut buf = [0; 1500];
        let (amt, src) = socket.recv_from(&mut buf)?;
        let received = String::from_utf8_lossy(&buf[..amt]);

        if received.contains("INVITE") {
            println!("Received INVITE from: {}", src);

            // Send 200 OK
            let response = "SIP/2.0 200 OK\r\n\r\n";
            socket.send_to(response.as_bytes(), &src)?;
            println!("Sent 200 OK");

            // Sleep briefly to simulate call setup
            thread::sleep(Duration::from_millis(500));

            // Open the WAV file
            let mut file = File::open("meatbag.wav")?;
            let mut wav_data = Vec::new();
            file.read_to_end(&mut wav_data)?;

            println!("Sending audio...");

            // "Stream" WAV data — simulate sending via RTP
            // In reality, RTP headers should be here, but for testing, we just blast raw audio
            for chunk in wav_data.chunks(160) { // 20ms @ 8kHz PCM
                socket.send_to(chunk, &src)?;
                thread::sleep(Duration::from_millis(20));
            }

            println!("Finished sending audio.");

            // After sending, send BYE
            let bye = "BYE sip:voicebbs@client SIP/2.0\r\n\r\n";
            socket.send_to(bye.as_bytes(), &src)?;
            println!("Sent BYE to {}", src);
        }
    }
}