set RUSTFLAGS=-Awarnings
set RUST_BACKTRACE=1
cargo build
"./target/debug/gila.exe" --mode %1 --file %2