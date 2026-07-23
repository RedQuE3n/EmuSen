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
            string logDir = Path.Combine(Directory.GetCurrentDirectory(), "Logs");
            Directory.CreateDirectory(logDir); // no-op if it already exists
            // Timestamped per launch (not a fixed "console.log") so a new
            // session doesn't overwrite the previous one's log - each capture
            // sent for investigation is now self-identifying by filename
            // alone, without needing to check the upload time separately.
            string logPath = Path.Combine(logDir, $"console_{DateTime.Now:yyyyMMdd_HHmmss}.log");
            StreamWriter fileWriter = new StreamWriter(logPath, append: false) { AutoFlush = true };
            Console.SetOut(new TeeTextWriter(originalOut, fileWriter));

            // Pick the ROM to run: first command-line argument if given (e.g.
            // `dotnet run -- /path/to/game.smc`), otherwise fall back to the
            // hardcoded default below. Raylib itself has no built-in file-picker
            // dialog, so a CLI arg is the standard way to make this pickable for
            // a console-launched build - the Avalonia frontend (EmuSen.Frontend)
            // has a real Open ROM... file picker instead, if that's preferred.
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

                // Drives the whole emulation - CPU/PPU/APU/memory, the
                // per-scanline timing loop, save states, SRAM. See
                // Cores/ICore.cs and this class's own header comment for
                // why this exists instead of Main owning that loop
                // directly the way it used to (that loop is now the ONE
                // copy shared with EmulatorSession/the Avalonia frontend,
                // not a second hand-maintained copy here).
                VenusCore core = new VenusCore(headless: false);
                core.LoadRom(romPath);
                DebugSettings.CpuVerboseLogging = false;
                DebugSettings.Spc700VerboseLogging = false;

                // The reusable debug toolchain - see Debug/IDebugTarget.cs
                // and Debug/DebugCommandProcessor.cs. Built once here so
                // both the F1 dump below and the new F4 interactive prompt
                // go through the exact same underlying data, rather than
                // each hotkey reaching into cpu/bus/ppu on its own.
                SnesDebugTarget debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!);
                DebugCommandProcessor debugCmd = new DebugCommandProcessor(debugTarget);

                // Continuous version of the F3 screenshot utility - see
                // Debug/FrameRecorder.cs. Core-agnostic itself; only the
                // capture callback wired in at the F6 hotkey below is
                // Raylib-specific.
                FrameRecorder frameRecorder = new FrameRecorder(debugTarget);

                // Yoshi/coin WRAM-staging investigation: registered here
                // instead of via the F4 prompt so it's active from the very
                // first CPU instruction, not from whenever a human can
                // press F4 after the window opens (hundreds of frames in).
                // Every prior trace of this range started after boot; this
                // rules out "the population happens very early and we keep
                // missing it." Remove once the investigation concludes.
                debugTarget.Watches.AddWatch("WRAM", 0x8000, 0x1800);

                // Same investigation, one level upstream: a ground-truth CPU
                // trace found the real DMA-trigger routine at $00A300 (not
                // $00A317, which was just partway through it) reads a job
                // table at $0D80-$0D9F (source pointer + pending-item counts
                // for both a palette-upload branch and a graphics-upload
                // branch) but never writes it - so whatever queues a job
                // (and would determine whether Yoshi's slot points at valid
                // WRAM data) does so somewhere else entirely. Zero writes to
                // this range showed up in a 20480-instruction ground-truth
                // trace that otherwise spanned many frames of active
                // dispatching, meaning this table is very likely populated
                // once, before that trace started - same shape as the
                // original $8000-$97FF finding. Watching from power-on
                // closes that gap here too.
                debugTarget.Watches.AddWatch("WRAM", 0x0D80, 0x0080);

                while (core.Renderer!.IsOpen())
                {
                    core.RunFrame();

                    // Poll real keyboard/gamepad state once per frame and feed it
                    // into the emulated controller. This has to happen before the
                    // next RunFrame() call (i.e. before that frame's own
                    // LatchAutoJoypad), so doing it here - right after a frame
                    // completes - keeps the timing correct. The actual key/pad
                    // mapping lives in InputBindings.cs, not here.
                    InputBindings.ApplyInput(core.Bus!);

                    core.Renderer.DrawFrame(core.Bus!, core.TotalFrames);

                    // All debug/dev hotkeys (F1-F6, F9, P) live in one place -
                    // see RunHotkeys below. Kept separate from the per-frame
                    // loop above so the actual emulation driving isn't buried
                    // under debug UI dispatch.
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

        // Everything triggered by a keypress or a bounded background trace,
        // checked once per completed frame. Deliberately separate from
        // Main's per-frame loop - that loop is the actual emulation driver
        // and reads much more clearly without ~170 lines of debug-hotkey
        // dispatch interleaved into it. Nothing here changes emulation
        // behavior; it's read-only inspection plus the F5/F9 save-state
        // and F3 screenshot side effects, same as before the extraction.
        private static void RunHotkeys(
            VenusCore core, SnesDebugTarget debugTarget, DebugCommandProcessor debugCmd, FrameRecorder frameRecorder,
            string statePath)
        {
            // Called every completed frame, recording or not - CaptureFrame
            // is a cheap no-op when IsRecording is false, so no gating
            // needed here. Has to run unconditionally like this (not just
            // on a keypress) so every frame while recording gets a chance
            // to be captured, not only the frame F6 happened to be pressed
            // on.
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
                    string dir = frameRecorder.Start(Path.Combine("Logs", "Recordings"));
                    Console.WriteLine($"[RECORD] Started -> {dir}");
                }
            }

            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.P))
            {
                _bgScrollTrace.Start(300);
                DebugSettings.AllScrollWriteLogging = true;
                Console.WriteLine("[BG SCROLL] --- Starting 300-frame scroll trace ---");
            }

            // On-demand full CPU+PPU snapshot - see StateDump.cs.
            // Formatted to be directly comparable to MesenCE's own
            // Status panel, since that's exactly what got
            // hand-transcribed from screenshots repeatedly during
            // the scroll-jitter investigation. Now goes through
            // debugTarget.GetSummaryText() (which itself just
            // delegates to StateDump.DumpAll - nothing lost),
            // rather than calling StateDump directly, so this
            // hotkey uses the same toolchain the new F4 prompt
            // does instead of its own separate path.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F1))
            {
                Console.WriteLine(debugTarget.GetSummaryText());
            }

            // Dumps every active OAM sprite's exact X/Y/tile/attr -
            // ground truth for checking whether an apparent
            // "duplicate sprite" is really the same OAM entry
            // rendered more than once, or genuinely separate OAM
            // entries (i.e. a game-logic question, not a
            // rendering bug). Left calling renderer.DumpActiveOam
            // directly rather than routing through the new
            // toolchain - that method also returns rects used
            // elsewhere (the "O" key's overlay), a second
            // responsibility SnesDebugTarget.GetSprites()
            // deliberately doesn't take on. The new `sprites`
            // command (via F4) is the generalized equivalent of
            // just this dump's printed part, going forward.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F2))
            {
                core.Renderer!.DumpActiveOam(core.Bus!.Ppu);
            }

            // Interactive debug command prompt - see
            // Debug/DebugCommandProcessor.cs. Blocking by design:
            // there's no way to pause emulation mid-frame yet (see
            // that file's header comment), so this just blocks the
            // console for as long as it takes to type commands,
            // the same way the F1/F2 hotkeys already block for the
            // instant it takes to print their output - just for
            // longer, and interactively. Type 'help' for the
            // command list, 'exit' or an empty line to resume.
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

            // Saves the current frame as a PNG. Added because the
            // coin investigation hit a wall that console-log text
            // alone can't resolve any further: the DMA/VRAM trace
            // confirmed real, changing tile pixel data is landing
            // at $C000, which means the upload path looks healthy
            // - so the next question ("is this even the coin
            // tile, and what does the screen actually look like
            // where a coin should be") needs an actual image, not
            // more log analysis.
            //
            // Also writes a companion .txt with the same frame
            // count (plus a wall-clock timestamp) so the PNG can
            // be directly and reliably cross-referenced against
            // console.log lines that mention the same frame
            // number, without depending on line-proximity
            // guessing. Uses debugTarget.CoreName/FrameCount
            // rather than anything SNES-specific, so this stays
            // correct unchanged if a future core is swapped in.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F3))
            {
                Directory.CreateDirectory("Logs");
                string baseName = $"screenshot_frame{debugTarget.FrameCount}";
                string shotPath = Path.Combine("Logs", baseName + ".png");
                string metaPath = Path.Combine("Logs", baseName + ".txt");

                Raylib_cs.Raylib.TakeScreenshot(shotPath);
                File.WriteAllText(metaPath,
                    $"Core: {debugTarget.CoreName}\n" +
                    $"Frame: {debugTarget.FrameCount}\n" +
                    $"Wall-clock: {DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}\n");

                Console.WriteLine($"[SCREENSHOT] Saved {shotPath} (Core={debugTarget.CoreName} Frame={debugTarget.FrameCount})");
            }

            // Save states (F5 save, F9 load) - one slot per ROM, named to
            // match its .srm save. Both now go straight through
            // VenusCore.SaveState/LoadState (via ICore) rather than each
            // frontend hand-rolling its own BinaryWriter/StateSerializer
            // dance - this used to be duplicated against EmulatorSession's
            // near-identical version.
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
