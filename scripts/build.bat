set RUSTFLAGS=-Awarnings
set RUST_BACKTRACE=1
cargo build --release
cd gila_socket
cargo build --release
cd ../gila_time
cargo build --release