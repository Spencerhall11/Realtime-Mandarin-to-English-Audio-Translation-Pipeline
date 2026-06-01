import socket
import struct
import numpy as np
import torch
from transformers import pipeline, MarianTokenizer, MarianMTModel

# Load model to GPU if possible
device = 0 if torch.cuda.is_available() else -1
print(head := f"Loading model on {'GPU (CUDA)' if device == 0 else 'CPU'}")

# Speech to text pipeline 
asr_pipe = pipeline(
    "automatic-speech-recognition",
    model="openai/whisper-base",
    device=device
)

# Explicitly load translation components to bypass the strict pipeline registry
print("Loading translation model components from local cache...")
trans_tokenizer = MarianTokenizer.from_pretrained("Helsinki-NLP/opus-mt-zh-en")
trans_model = MarianMTModel.from_pretrained("Helsinki-NLP/opus-mt-zh-en")

if device == 0:
    trans_model = trans_model.to("cuda")

# Bind to local TCP socket
server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.bind(('127.0.0.1', 5005))
server.listen(1)
print("ML backend engine fully initialized and listening on port 5005...")

conn, addr = server.accept()
print(f"Connected by frontend client at: {addr}")

#ambient noise values
ambient_noise_floor=0.002
alpha = 0.95

try:
    while True:
        # Read payload header safely to capture sudden client drops
        try:
            header = conn.recv(4)
            if not header: 
                print("Client disconnected gracefully.")
                break
            data_len = struct.unpack('!I', header)[0]
        except (ConnectionResetError, ConnectionAbortedError):
            print("\n[Socket] Connection reset by client application window exit.")
            break

        # Read audio bytes
        raw_data = b""
        while len(raw_data) < data_len:
            packet = conn.recv(data_len - len(raw_data))
            if not packet: 
                break
            raw_data += packet

        # Break if socket closes mid packet
        if len(raw_data) < data_len:
            break
        
        # Convert bytes to array
        audio_np = np.frombuffer(raw_data, dtype=np.float32).copy()
        english_translation = " . . . "

        try:
            # Voice Activity Detection Gate 
            rms_energy = np.sqrt(np.mean(audio_np**2)) if len(audio_np) > 0 else 0.0

            #calculate based on environment
            dynamic_gate = ambient_noise_floor * 2.5
            
            # Baseline cutoff for background mic hiss/breathing noise
            if rms_energy < dynamic_gate:
                ambient_noise_floor = (alpha * ambient_noise_floor) + ((1.0 - alpha) * rms_energy)
                english_translation = " . . . "

            else:
                #Transcribe speech using decoding constraints
                asr_result = asr_pipe(
                    {"sampling_rate": 16000, "raw": audio_np},
                    generate_kwargs={
                        "language": "zh",                    
                        "task": "transcribe",                
                        "temperature": 0.0,                  
                        "return_timestamps": False    
                    }
                )
                mandarin_text = asr_result["text"].strip()

                # Guard against repetitive loops ("嗯嗯嗯...")
                if len(mandarin_text) > 5:
                    unique_ratio = len(set(mandarin_text)) / len(mandarin_text)
                    # If unique characters make up less than 20% of a long string, it's a hallucination
                    is_hallucination_loop = unique_ratio < 0.20
                else:
                    is_hallucination_loop = False


                if mandarin_text and not is_hallucination_loop:
                    print(f"Transcribed (ZH): {mandarin_text} (Energy: {rms_energy:.4f})", flush=True)
                    
                    # Tokenize the incoming Mandarin string text
                    inputs = trans_tokenizer(mandarin_text, return_tensors="pt")
                    if device == 0:
                        inputs = {k: v.to("cuda") for k, v in inputs.items()}
                        
                    # Run forward pass generation inside a no_grad context to save VRAM
                    with torch.no_grad():
                        generated_ids = trans_model.generate(**inputs)
                        
                    # Decode the generated tensor tokens back into an English string
                    english_translation = trans_tokenizer.decode(generated_ids[0], skip_special_tokens=True)
                    print(f"Translated (EN): {english_translation}",flush=True)

                    ambient_noise_floor *=0.99
                else:
                    english_translation = " . . . "

        except Exception as e:
            print(f"Inference error: {e}")
            english_translation = "[Error processing audio]"

        # Send final translated English string to Rust
        try:
            conn.sendall(english_translation.encode('utf-8') + b'\n')
        except (ConnectionResetError, ConnectionAbortedError):
            break

finally:
    print("Closing connection descriptors cleanly.")
    conn.close()
    server.close()