use std::io::{Write, BufReader, BufRead};
use std::net::TcpStream;
use byteorder::{BigEndian, WriteBytesExt};
use crossbeam_channel::unbounded;

mod audio;
use audio::AudioRecorder;

// Audio context
const SAMPLE_RATE: u32 = 16000;
const CHUNK_SIZE: usize = (SAMPLE_RATE as f32 * 1.5) as usize; 

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Connect to Python ML Engine
   println!("Connecting to Python translation server on 127.0.0.1:5005...");
    let mut stream = TcpStream::connect("127.0.0.1:5005")
        .expect("Failed to connect to Python server. Make sure server.py is running first!");
    let mut reader = BufReader::new(stream.try_clone()?);
    println!("Connected successfully!");

    // Set up the crossbeam ring channel
    let (tx, rx) = unbounded::<f32>();

    //Launch background hardware loop
    let recorder = AudioRecorder::new(SAMPLE_RATE);
    recorder.spawn_capture_loop(tx)
        .expect("Failed to initialize background audio capture pipeline");

    // Track frame aggregation 
    let mut audio_buffer: Vec<f32> = Vec::with_capacity(CHUNK_SIZE);
    println!("\n>>> Listening... Speak into your microphone now. <<<\n");
    for sample in rx {
        audio_buffer.push(sample);

        if audio_buffer.len() >= CHUNK_SIZE {
            // Swap out the full buffer contents into an isolated variable
            let processing_chunk = std::mem::take(&mut audio_buffer);

            // Safe transmuteless cast of f32 vector into a raw byte slice
            let raw_bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    processing_chunk.as_ptr() as *const u8,
                    processing_chunk.len() * 4,
                )
            };

            let mut header = Vec::new();
            header.write_u32::<BigEndian>(raw_bytes.len() as u32)?;

            // Fixed scoping brackets and error logic paths
            if stream.write_all(&header).is_ok() && stream.write_all(raw_bytes).is_ok() {
                stream.flush()?;

                let mut response = String::new();
                if reader.read_line(&mut response).is_ok() {
                    let clean_text = response.trim();
                    if clean_text != " . . . " && !clean_text.is_empty() {
                        println!("Translation: {}", clean_text);
                    }
                }
            }
        }
    }

    Ok(())
}