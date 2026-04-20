@echo off
cd /d "%~dp0"
taskkill /F /IM node.exe >nul 2>&1
taskkill /F /IM cargo.exe >nul 2>&1
call npm install
call npm run tauri dev