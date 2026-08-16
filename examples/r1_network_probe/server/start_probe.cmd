@echo off
setlocal
pushd "%~dp0"
start "R1 probe server" /b "%~dp0samp-server.exe"
timeout /t 2 /nobreak >nul
start "R1 probe NPC" /b "%~dp0samp-npc.exe" -h 127.0.0.1 -p 7777 -n R1ProbeBot -m r1_probe_bot
popd
endlocal
