set "THIS_PATH=%~dp0"
set "THIS_PATH=%THIS_PATH:~0,-1%"

cargo build --release

cmd /c %THIS_PATH%/../../target/release/pac_file_http_server.exe 127.0.0.1:6001 %THIS_PATH%/example.pac

cmd
