use std::net::UdpSocket;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

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
            payload_type: 0, // Payload type 0 = PCMU (G.711 u-Law)
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

            // RTP state
            let mut sequence_number = 0u16;
            let mut timestamp = 0u32;
            let ssrc = 0x12345678; // Random SSRC

            // Stream WAV data in 20ms chunks (~160 bytes at 8kHz)
            for chunk in wav_data.chunks(160) {
                let header = RtpHeader::new(sequence_number, timestamp, ssrc);
                let mut packet = header.build();
                packet.extend_from_slice(chunk);

                socket.send_to(&packet, &src)?;

                sequence_number = sequence_number.wrapping_add(1);
                timestamp = timestamp.wrapping_add(160); // 160 samples = 20ms at 8000Hz

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