# Realtime-Mandarin-to-English-Audio-Translation-Pipeline
A low-latency, dual-language audio utility that captures live hardware microphone inputs, optimizes the acoustic stream, and executes local deep learning transformers to stream translated English text in real time. Bridges a native Rust audio frontend with an adaptive, unbuffered PyTorch inference server in Python over local TCP sockets.


A local, low-latency utility that streams hardware mic input from a native Rust client to a Python PyTorch server over TCP sockets to handle Mandarin automatic speech recognition (ASR) and translation (NMT)

setUp
git clone [https://github.com/Spencerhall11/realtime-mandarin-english-translation-pipeline.git](https://github.com/Spencerhall11/realtime-mandarin-english-translation-pipeline.git)
cd realtime-mandarin-english-translation-pipeline
Run
.\run.bat
