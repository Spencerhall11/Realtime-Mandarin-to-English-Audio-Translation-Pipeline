@echo off
title Real-Time Audio Translator 
echo    Launching Real-Time Mandarin Audio Translator

:: Boot up the Python Machine Learning Server
echo [SYSTEM] Spinning up Python ML Backend Server...
:: 1. Dynamic Virtual Environment Detection & Backend Launch
echo [SYSTEM] Locating virtual environment and spinning up Python ML Server...

if exist ".\python_ml_server\.venv\Scripts\activate.bat" (
    :: Found nested inside the server folder
    start "Python ML Server Backend" cmd /k "cd python_ml_server && .venv\Scripts\activate.bat && python -u server.py"
) else if exist ".\.venv\Scripts\activate.bat" (
    :: Found at the root directory level
    start "Python ML Server Backend" cmd /k ".\.venv\Scripts\activate.bat && cd python_ml_server && python -u server.py"
) else if exist ".\python_ml_server\venv\Scripts\activate.bat" (
    :: Found nested but named 'venv' without the dot
    start "Python ML Server Backend" cmd /k "cd python_ml_server && venv\Scripts\activate.bat && python -u server.py"
) else (
    :: Fallback: Try running with system python if no venv is isolated
    echo [WARNING] No isolated virtual environment found! Attempting global python launch...
    start "Python ML Server Backend" cmd /k "cd python_ml_server && python -u server.py"
)



:: Intentional delay to let models bind to GPU
echo [SYSTEM] Pausing for 12 seconds to allow Whisper and OPUS weights to initialize...
echo          (If this is your first run, wait for downloads to complete in the other window before retrying)
timeout /t 12 /nobreak > nul

:: Launch the optimized Rust client in the current terminal window
echo [SYSTEM] Initializing Hardware Audio Capture Frontend...
echo -------------------------------------------------------
cd rust_audio_client
cargo run --release

pause