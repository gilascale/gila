set RUSTFLAGS=-Awarnings
set RUST_BACKTRACE=1
cargo build --release
"./target/release/gila.exe" --mode %1 --file %2