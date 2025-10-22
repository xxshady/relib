echo "[run.sh] running cargo build:"
cargo build &&

# need to copy the host binary because we are running `cargo build` in the host binary
# and it will try to write to host file
if [[ "$OSTYPE" == "win32" || "$OSTYPE" == "msys"  ]]; then
  echo "[run.sh] copying host.exe" &&
  cp target/debug/host.exe target/debug/copy_host.exe &&
  echo "[run.sh] running copied host:" &&
  ./target/debug/copy_host.exe
else
  echo "[run.sh] copying host" &&
  cp target/debug/host target/debug/copy_host &&
  echo "[run.sh] running copied host:" &&
  ./target/debug/copy_host
fi

