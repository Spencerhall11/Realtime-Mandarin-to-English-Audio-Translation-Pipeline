use std::io::{Write, BufReader, BufRead};
use std::net::TcpStream;
use byteorder::{BigEndian, WriteBytesExt};

pub struct InferenceClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl InferenceClient {
    /// Connects to the local Python ML server socket descriptor
    pub fn connect(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(address)?;
        // Clone the stream handle cleanly to decouple read and write operations
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Self { stream, reader })
    }

    /// Packs and streams a raw f32 audio buffer over the socket network connection
    pub fn stream_and_translate(&mut self, audio_chunk: &[f32]) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Zero-copy pointer cast of f32 segment slices to system byte boundaries
        let raw_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                audio_chunk.as_ptr() as *const u8,
                audio_chunk.len() * 4, // 4 bytes per float32
            )
        };

        // Serialize packet network length header (Big Endian)
        let mut header = Vec::with_capacity(4);
        header.write_u32::<BigEndian>(raw_bytes.len() as u32)?;

        // Ship header and raw byte payload down the TCP stream connection pipe
        self.stream.write_all(&header)?;
        self.stream.write_all(raw_bytes)?;
        self.stream.flush()?;

        // Wait for synchronous translation response string line break
        let mut response = String::new();
        self.reader.read_line(&mut response)?;
        
        let clean_text = response.trim().to_string();
        if clean_text != " . . . " && !clean_text.is_empty() {
            Ok(Some(clean_text))
        } else {
            Ok(None)
        }
    }
}