# Macro Recorder - Made with Rust 🦀

A powerful macro recording and playback application built with Rust, featuring global hooks and precise timing control.

## Features

- ✅ **Global Hooks**: Uses Windows API for system-wide event hooking
- ✅ **Recording**: Captures mouse movements, clicks, and keyboard input
- ✅ **Playback**: Replays macros with accurate timing
- ✅ **Hotkeys**: Ctrl+R (Record), Ctrl+P (Pause/Resume), Ctrl+Q (Stop)
- ✅ **Pause/Resume**: Pause and resume recording/playback functionality
- ✅ **File Format**: Saves as macro files (human-readable text format, .mcr for short)

## System Requirements

- Windows 10/11
- Rust 1.70+ (for building from source)
- Administrator privileges (recommended)

## Installation & Running

### Method 1: Build and run directly
```bash
cargo run --release
```
### Method 2: Build executable
```bash
cargo build --release
```

**Note**: Running with Administrator privileges is recommended for optimal global hook functionality.

## Usage

### Recording
1. Click "● Record" button or press `Ctrl+R`
2. Perform the mouse/keyboard actions you want to record
3. Press `Ctrl+P` to pause/resume recording
4. Press `Ctrl+Q` to stop and save the macro file

### Playback
1. Click "▶ Play" button or open a .mcr file
2. The macro will be replayed with accurate timing
3. Use `Ctrl+P` to pause/resume, `Ctrl+Q` to stop

### Global Hotkeys
- `Ctrl+R`: Start recording
- `Ctrl+P`: Pause/resume (during recording or playback)
- `Ctrl+Q`: Stop current session

## File Format

The .mcr files use a simple text format:
```
timestamp;event_type;parameters
0.000000;KDOWN;char=h
0.100000;KUP;char=h
0.200000;MMOVE;x=100;y=200
0.300000;MDOWN;button=left;x=100;y=200
0.400000;MUP;button=left;x=100;y=200
```

## Troubleshooting

### Global hooks not working
- Run the application with Administrator privileges
- Check that antivirus software isn't blocking the application
- Ensure no other applications are hooking global events

### Build failures
- ~~mf do you even have rust~~
- Try to install [Rust](https://www.rust-lang.org/tools/install) and build the project again


## Development

### Project Structure
```
MacroRecorder/
├── src/
│   ├── main.rs                 # Main GUI application
│   ├── hooks.rs                # Windows API global hooks
│   ├── events.rs               # Event system & serialization
│   ├── recorder.rs             # Recording logic
│   └── player.rs               # Playback logic
├── Cargo.toml                  # Rust project configuration
├── demo.mcr                    # Demo macro file
└── README.md                   # Documentation
```


### Development Commands
```bash
# Run in debug mode
cargo run

# Run optimized build
cargo run --release

# Run linter
cargo clippy

# Format code
cargo fmt

# Run tests
cargo test
```

## License

MIT License - Free to use and modify.


## FAQ

### Why Rust?
I tried python and pynput, but it was slow ~~and i dont know how to implement Windows hooks in python~~. Also, Rust also have a GUI framework, so i can make a GUI for this project. C++ GUI on the other hand, is too complex.

### How does this things work?

Using Windows API to hook global events, and then record the events and save them to a file. Workflow is as follows:
1. User click "Record" button or press `Ctrl+R`
2. The application will hook global events using `SetWindowsHookExA` and `SetWindowsHookExW`
3. The application will record the events (with function `GetAsyncKeyState` and `GetAsyncKeyStateEx`) and save them to a file using `WriteFile`
4. The application will save the file to a folder which the user can choose.

### Can this run on other OS?
~~Do they have Windows API? No they don't (right?).~~
<br/>The answer is no.

### Why do I make this?
My Macro Recorder free trial ran out.
