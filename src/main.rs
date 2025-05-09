use std::net::UdpSocket;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::{Duration, Instant};

use log::{info, warn, error, debug};
use env_logger;

#[derive(Debug)]
struct RtpHeader {
    version: u8,
    padding: bool,
    extension: bool,
    csrc_count: u8,
    marker: bool,
    payload_type: u8,
    sequence_number: u16,
    timestamp: u32,
    ssrc: u32,
}

impl RtpHeader {
    fn new(sequence_number: u16, timestamp: u32, ssrc: u32) -> Self {
        RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 0, // G.711 μ-law (PCMU)
            sequence_number,
            timestamp,
            ssrc,
        }
    }

    fn build(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12);

        let b0 = (self.version << 6)
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | (self.csrc_count & 0x0F);
        buf.push(b0);

        let b1 = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);
        buf.push(b1);

        buf.extend_from_slice(&self.sequence_number.to_be_bytes());
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.ssrc.to_be_bytes());

        buf
    }
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let socket = UdpSocket::bind("0.0.0.0:5060")?;
    info!("VoiceBBS SIP server listening on UDP 5060");

    loop {
        let mut buf = [0; 1500];
        let (amt, src) = socket.recv_from(&mut buf)?;
        let received = String::from_utf8_lossy(&buf[..amt]);

        if received.contains("INVITE") {
            info!("Received INVITE from: {}", src);

            // --- Prepare 200 OK with SDP ---
            let your_ip = "155.138.203.121"; // Replace with your VPS public IP

            let sdp = format!(
                "v=0\r\n\
o=- 0 0 IN IP4 {ip}\r\n\
s=VoiceBBS\r\n\
c=IN IP4 {ip}\r\n\
t=0 0\r\n\
m=audio 8000 RTP/AVP 0 101\r\n\
a=rtpmap:0 PCMU/8000\r\n\
a=rtpmap:101 telephone-event/8000\r\n\
a=fmtp:101 0-15\r\n",
                ip = your_ip
            );

            let response = format!(
                "SIP/2.0 200 OK\r\n\
Via: SIP/2.0/UDP {src}\r\n\
Contact: <sip:{ip}:5060>\r\n\
Content-Type: application/sdp\r\n\
Content-Length: {}\r\n\
Allow: INVITE, ACK, CANCEL, OPTIONS, BYE\r\n\
Supported: timer\r\n\
Session-Expires: 1800;refresher=uac\r\n\
\r\n\
{}",
                sdp.len(),
                sdp,
                ip = your_ip,
                src = src.ip()
            );

            socket.send_to(response.as_bytes(), &src)?;
            info!("Sent 200 OK with SDP");

            // --- Wait for SIP ACK ---
            info!("Waiting for ACK from {}", src);
            let mut ack_received = false;
            let start_time = Instant::now();

            socket.set_read_timeout(Some(Duration::from_secs(1)))?;

            while start_time.elapsed() < Duration::from_secs(5) {
                let mut ack_buf = [0; 1500];
                match socket.recv_from(&mut ack_buf) {
                    Ok((ack_amt, ack_src)) => {
                        if ack_src.ip() == src.ip() {
                            let ack_msg = String::from_utf8_lossy(&ack_buf[..ack_amt]);
                            if ack_msg.contains("ACK") {
                                info!("Received ACK from {}", ack_src);
                                ack_received = true;

                                // RESET TIMEOUT after ACK
                                socket.set_read_timeout(None)?;

                                break;
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Timeout occurred, continue waiting
                        continue;
                    }
                    Err(e) => {
                        error!("Error receiving ACK: {}", e);
                        break;
                    }
                }
            }

            if !ack_received {
                warn!("No ACK received, skipping sending audio!");
                // RESET TIMEOUT in case of failure
                socket.set_read_timeout(None)?;
                continue; // Skip this call if no ACK
            }

            // --- Now send the audio (meatbag.wav) ---
            let mut file = File::open("meatbag.wav")?;
            let mut wav_data = Vec::new();
            file.read_to_end(&mut wav_data)?;

            info!("Sending audio...");

            let mut sequence_number = 0u16;
            let mut timestamp = 0u32;
            let ssrc = 0x12345678;

            for chunk in wav_data.chunks(160) {
                let header = RtpHeader::new(sequence_number, timestamp, ssrc);
                let mut packet = header.build();
                packet.extend_from_slice(chunk);

                socket.send_to(&packet, &src)?;

                sequence_number = sequence_number.wrapping_add(1);
                timestamp = timestamp.wrapping_add(160);

                thread::sleep(Duration::from_millis(20));
            }

            info!("Finished sending audio.");

            // --- Send SIP BYE cleanly ---
            let bye = "BYE sip:voicebbs@client SIP/2.0\r\n\r\n";
            socket.send_to(bye.as_bytes(), &src)?;
            info!("Sent BYE to {}", src);
        }
    }
}