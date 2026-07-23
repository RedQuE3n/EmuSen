using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Debug;
using EmuSen.Bindings;

namespace EmuSen.Frontend
{
    class Program
    {
        private const int StatusEveryNFrames = 60;

        private static readonly DebugTools.BoundedTrace _bgScrollTrace = new();

        static void Main(string[] args)
        {
            TextWriter originalOut = Console.Out;

            // ROM path resolution - see EmuSen_Frontend_Driver.md §1.
            string defaultRomPath = "/home/red/Documents/Roms/SMW.smc"; // temporary location
            string romPath = args.Length > 0 ? args[0] : defaultRomPath;

            if (!File.Exists(romPath))
            {
                Console.WriteLine($"[ERROR] ROM not found: {romPath}");
                Console.WriteLine("Usage: dotnet run -- <path-to-rom.smc>");
                return;
            }
            Console.WriteLine($"[ROM] Loading: {romPath}");

            try
            {
                string statePath = Path.Combine(Directory.GetCurrentDirectory(), "Saves", Path.GetFileNameWithoutExtension(romPath) + ".state");

                // VenusCore drives the whole emulation - see EmuSen_Frontend_Driver.md §1.
                VenusCore core = new VenusCore(headless: false);

                // Log dir setup - deliberately placed here, not earlier. See
                // EmuSen_Frontend_Driver.md §1.
                string logDir = Path.Combine(Directory.GetCurrentDirectory(), "Logs", core.CoreName, $"console_{DateTime.Now:yyyyMMdd_HHmmss}");
                Directory.CreateDirectory(logDir);
                Console.SetOut(new CategorizedLogWriter(originalOut, logDir));

                core.LoadRom(romPath);
                DebugSettings.CpuVerboseLogging = false;
                DebugSettings.Spc700VerboseLogging = false;

                // Debug toolchain, built once - see EmuSen_Frontend_Driver.md §1.
                SnesDebugTarget debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!);
                DebugCommandProcessor debugCmd = new DebugCommandProcessor(debugTarget);

                FrameRecorder frameRecorder = new FrameRecorder(debugTarget);

                // Power-on watch registration (Yoshi/coin investigation) -
                // see EmuSen_Frontend_Driver.md §1.
                debugTarget.Watches.AddWatch("WRAM", 0x8000, 0x1800);
                debugTarget.Watches.AddWatch("WRAM", 0x0D80, 0x0080);

                while (core.Renderer!.IsOpen())
                {
                    core.RunFrame();

                    // Must happen before the next RunFrame()'s own
                    // LatchAutoJoypad - see EmuSen_Frontend_Driver.md §1.
                    InputBindings.ApplyInput(core.Bus!);

                    core.Renderer.DrawFrame(core.Bus!, core.TotalFrames);

                    // All debug/dev hotkeys - see EmuSen_Frontend_Driver.md §2.
                    RunHotkeys(core, debugTarget, debugCmd, frameRecorder, statePath);
                }
                core.SaveSram(); // final flush on clean exit
                core.Renderer.Shutdown();
            }
            catch (Exception ex)
            {
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
            }
        }

        // Every debug/dev hotkey - see EmuSen_Frontend_Driver.md §2.
        private static void RunHotkeys(
            VenusCore core, SnesDebugTarget debugTarget, DebugCommandProcessor debugCmd, FrameRecorder frameRecorder,
            string statePath)
        {
            // Every frame, cheap no-op when not recording - see
            // EmuSen_Frontend_Driver.md §2.
            frameRecorder.CaptureFrame(path => Raylib_cs.Raylib.TakeScreenshot(path));

            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F6))
            {
                if (frameRecorder.IsRecording)
                {
                    Console.WriteLine($"[RECORD] Stopped: {frameRecorder.SessionDir}");
                    frameRecorder.Stop();
                }
                else
                {
                    string dir = frameRecorder.Start(Path.Combine("Logs", debugTarget.CoreName, "Recordings"));
                    Console.WriteLine($"[RECORD] Started -> {dir}");
                }
            }

            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.P))
            {
                _bgScrollTrace.Start(300);
                DebugSettings.AllScrollWriteLogging = true;
                Console.WriteLine("[BG SCROLL] --- Starting 300-frame scroll trace ---");
            }

            // Full CPU+PPU snapshot - see EmuSen_Frontend_Driver.md §2.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F1))
            {
                Console.WriteLine(debugTarget.GetSummaryText());
            }

            // OAM sprite dump - see EmuSen_Frontend_Driver.md §2.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F2))
            {
                core.Renderer!.DumpActiveOam(core.Bus!.Ppu);
            }

            // Interactive debug prompt - see EmuSen_Frontend_Driver.md §2.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F4))
            {
                Console.WriteLine("--- Debug prompt (type 'help', 'exit' to resume) ---");
                while (true)
                {
                    Console.Write("debug> ");
                    string? line = Console.ReadLine();
                    if (line is null) break;
                    string trimmed = line.Trim();
                    if (trimmed.Length == 0 || trimmed.Equals("exit", StringComparison.OrdinalIgnoreCase) || trimmed.Equals("quit", StringComparison.OrdinalIgnoreCase))
                    {
                        break;
                    }
                    Console.WriteLine(debugCmd.Execute(trimmed));
                }
                Console.WriteLine("--- Resuming ---");
            }

            // Timestamped screenshot - see EmuSen_Frontend_Driver.md §2.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F3))
            {
                string coreLogDir = Path.Combine("Logs", debugTarget.CoreName);
                Directory.CreateDirectory(coreLogDir);
                string baseName = $"screenshot_frame{debugTarget.FrameCount}";
                string shotPath = Path.Combine(coreLogDir, baseName + ".png");
                string metaPath = Path.Combine(coreLogDir, baseName + ".txt");

                Raylib_cs.Raylib.TakeScreenshot(shotPath);
                File.WriteAllText(metaPath,
                    $"Core: {debugTarget.CoreName}\n" +
                    $"Frame: {debugTarget.FrameCount}\n" +
                    $"Wall-clock: {DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}\n");

                Console.WriteLine($"[SCREENSHOT] Saved {shotPath} (Core={debugTarget.CoreName} Frame={debugTarget.FrameCount})");
            }

            // Save/load state - see EmuSen_Frontend_Driver.md §2.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F5))
            {
                try
                {
                    core.SaveState(statePath);
                    Console.WriteLine($"[STATE] Saved: {statePath}");
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"[STATE] Save failed: {ex.Message}");
                }
            }
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F9))
            {
                try
                {
                    core.LoadState(statePath);
                    Console.WriteLine($"[STATE] Loaded: {statePath}");
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"[STATE] Load failed: {ex.Message}");
                }
            }
            if (_bgScrollTrace.ShouldLog())
            {
                Console.WriteLine($"[BG SCROLL] Frame {core.TotalFrames}: BG1 X={core.Bus!.Ppu.BgScrollX[0]} Y={core.Bus.Ppu.BgScrollY[0]}  |  BG2 X={core.Bus.Ppu.BgScrollX[1]} Y={core.Bus.Ppu.BgScrollY[1]}");
                if (!_bgScrollTrace.IsActive) DebugSettings.AllScrollWriteLogging = false;
            }

            if (core.TotalFrames % StatusEveryNFrames == 0)
            {
                Console.WriteLine(
                    $"[STATUS] Frame {core.TotalFrames} | PC=0x{core.Cpu!.PB:X2}{core.Cpu.PC:X4} | " +
                    $"TM={core.Bus!.Ppu.Tm:X2} BGMODE={core.Bus.Ppu.Bgmode:X2} INIDISP={core.Bus.Ppu.Inidisp:X2}"
                );
            }

            // Periodic SRAM autosave now happens inside VenusCore.RunFrame()
            // itself (see that class) - no longer duplicated here.
        }
    }
}
