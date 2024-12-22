set RUSTFLAGS=-Awarnings
set RUST_BACKTRACE=1
cargo build
cd gila_socket
cargo build
cd ../gila_time
cargo build