@echo off
rem Copied to \bin\<name>.cmd at publish time; the real app dir is \lib\EmuSen - see `man hier`.
rem Name-agnostic the same way launcher.sh is: %~n0 is this shim's own base name.
"%~dp0..\lib\EmuSen\%~n0.exe" %*
