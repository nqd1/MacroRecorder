@echo off
echo Building Rust Macro Recorder...

REM Build in release mode for better performance
cargo build --release

if %ERRORLEVEL% == 0 (
    echo Build successful!
    echo Running the application...
    echo.
    cargo run --release
) else (
    echo Build failed!
    pause
)
