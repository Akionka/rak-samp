@echo off
setlocal
pushd "%~dp0"
start "R5 probe server" /b "%~dp0samp-server.exe"
timeout /t 2 /nobreak >nul
start "R5 probe NPC" /b "%~dp0samp-npc.exe" -h 127.0.0.1 -p 7777 -n R5ProbeBot -m r5_probe_bot
popd
endlocal
