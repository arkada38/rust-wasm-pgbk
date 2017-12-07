@echo off

if not exist build mkdir build
if not exist src mkdir src

for /r %%a in (src\*.rs) do call :process "%%a"
goto :eof

:process
if "%~x1"==".rs" (
    rustc +nightly --target wasm32-unknown-unknown -O --crate-type=cdylib src\%~n1.rs -o build\%~n1.big.wasm
    wasm-gc build\%~n1.big.wasm build\%~n1.wasm
    del build\%~n1.big.wasm
)
