# Realtime-Mandarin-to-English-Audio-Translation-Pipeline
A low-latency, dual-language audio utility that captures live hardware microphone inputs, optimizes the acoustic stream, and executes local deep learning transformers to stream translated English text in real time. Bridges a native Rust audio frontend with an adaptive, unbuffered PyTorch inference server in Python over local TCP sockets.
