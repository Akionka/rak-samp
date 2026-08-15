@echo off
setlocal
pushd "%~dp0"
start "DL probe server" /b "%~dp0samp-server.exe"
timeout /t 2 /nobreak >nul
start "DL probe NPC" /b "%~dp0samp-npc.exe" -h 127.0.0.1 -p 7777 -n DLProbeBot -m dl_probe_bot
popd
endlocal
