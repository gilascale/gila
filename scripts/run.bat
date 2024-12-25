set RUSTFLAGS=-Awarnings
set RUST_BACKTRACE=1
cargo build
"./target/debug/gila.exe" --file "./example/test.gila" --mode "run"