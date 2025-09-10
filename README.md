# Macro Recorder - Rust Edition 🦀

High-performance macro recorder app giống Jitbit được viết bằng Rust, với khả năng theo dõi global keyboard và mouse events ngay cả khi app mất focus.

## Tính năng

- ✅ **Global Hooks**: Sử dụng Windows API để hook events toàn cục
- ✅ **Recording**: Ghi lại mouse movements, clicks, keyboard input
- ✅ **Playback**: Phát lại macro với timing chính xác
- ✅ **Hotkeys**: Ctrl+R (Record), Ctrl+P (Pause/Resume), Ctrl+Q (Stop)
- ✅ **Pause/Resume**: Tạm dừng và tiếp tục recording/playback
- ✅ **File Format**: Lưu dưới dạng .mcr (text format, dễ đọc)

## Yêu cầu hệ thống

- Windows 10/11
- Rust 1.70+ (để build từ source)
- Quyền Administrator (khuyến nghị)

## Cài đặt và chạy

### Cách 1: Build và chạy trực tiếp
```bash
cargo run --release
```

### Cách 2: Dùng build script
```bash
build_rust.bat
```

### Cách 3: Build executable
```bash
cargo build --release
# File .exe sẽ ở target/release/macro_recorder.exe
```

**Lưu ý**: Khuyến nghị chạy với quyền Administrator để global hooks hoạt động tốt nhất.

## Cách sử dụng

### Recording
1. Click "● Record" hoặc nhấn `Ctrl+R`
2. Thực hiện các thao tác mouse/keyboard muốn ghi lại
3. Nhấn `Ctrl+P` để tạm dừng/tiếp tục
4. Nhấn `Ctrl+Q` để dừng và lưu file

### Playback
1. Click "▶ Play" hoặc mở file .mcr
2. Macro sẽ được phát lại với timing chính xác
3. Sử dụng `Ctrl+P` để tạm dừng, `Ctrl+Q` để dừng

### Hotkeys toàn cục
- `Ctrl+R`: Bắt đầu recording
- `Ctrl+P`: Tạm dừng/tiếp tục (khi đang record/play)
- `Ctrl+Q`: Dừng session hiện tại

## File format

File .mcr sử dụng format text đơn giản:
```
timestamp;event_type;parameters
0.000000;KDOWN;char=h
0.100000;KUP;char=h
0.200000;MMOVE;x=100;y=200
0.300000;MDOWN;button=left;x=100;y=200
0.400000;MUP;button=left;x=100;y=200
```

## Troubleshooting

### Global hooks không hoạt động
- Chạy với quyền Administrator
- Kiểm tra antivirus không block ứng dụng
- Đảm bảo không có app khác đang hook global events

### Build thất bại
- Cài đặt Rust từ https://rustup.rs/
- Đảm bảo có Visual Studio Build Tools hoặc MinGW
- Chạy `cargo update` để cập nhật dependencies

### App không responsive
- Đây không nên xảy ra với Rust version
- Nếu có, hãy báo cáo bug với log details

## Tại sao chọn Rust?

### 🚀 **Performance vượt trội**
- ⚡ **Zero-cost abstractions** - Không overhead runtime
- 🧠 **Memory efficient** - Tự động quản lý memory, không GC
- 🎯 **Native speed** - Compile trực tiếp thành machine code
- 📦 **Single executable** - Không cần runtime dependencies

### 🔒 **An toàn và tin cậy**
- 🦀 **Memory safety** - Không buffer overflow, use-after-free
- 🔧 **Type safety** - Compiler catch bugs trước khi chạy
- 🛡️ **Thread safety** - Không data races
- 📊 **Predictable performance** - Không GC pauses

### 🎨 **Modern development experience**
- 🖼️ **egui** - Immediate mode GUI, responsive
- 📚 **Rich ecosystem** - Cargo package manager
- 🔄 **Hot reload** - Fast development cycle
- 📝 **Excellent tooling** - Built-in formatter, linter, docs

## Phát triển

### Cấu trúc dự án
```
MacroRecorder/
├── src/
│   ├── main.rs                 # Main GUI application
│   ├── hooks.rs                # Windows API global hooks
│   ├── events.rs               # Event system & serialization
│   ├── recorder.rs             # Recording logic
│   └── player.rs               # Playback logic
├── Cargo.toml                  # Rust project configuration
├── build_rust.bat              # Build helper script
├── demo.mcr                    # Demo macro file
└── README.md                   # Documentation
```

### Thêm tính năng mới
1. **Events**: Chỉnh sửa `hooks.rs` để thêm event types mới
2. **GUI**: Chỉnh sửa `main.rs` để thêm UI components
3. **Recording**: Chỉnh sửa `recorder.rs` để thêm recording logic
4. **Playback**: Chỉnh sửa `player.rs` để thêm playback features

### Development commands
```bash
# Development build với debug info
cargo run

# Release build tối ưu performance  
cargo run --release

# Check code quality
cargo clippy

# Format code
cargo fmt

# Run tests
cargo test
```

## License

MIT License - Free to use and modify.
