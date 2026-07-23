using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Memory;
using EmuSen.Apu;
using EmuSen.Processor;
using EmuSen.Video;
using EmuSen.Controllers;
using EmuSen.Debug;
using EmuSen.Bindings;

namespace EmuSen.Frontend
{
    class Program
    {
        private const int CyclesPerScanline = 227;
        private const int TotalScanlines = 262;
        private const int StatusEveryNFrames = 60;
        private const int SaveEveryNFrames = 300; // ~5 seconds at 60fps

        private static readonly DebugTools.BoundedTrace _bgScrollTrace = new();

        static void Main(string[] args)
        {
            TextWriter originalOut = Console.Out;
            string logDir = Path.Combine(Directory.GetCurrentDirectory(), "Logs");
            Directory.CreateDirectory(logDir); // no-op if it already exists
            string logPath = Path.Combine(logDir, "console.log");
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
                Cartridge cart = new Cartridge(romPath);
                string statePath = Path.Combine(Directory.GetCurrentDirectory(), "Saves", Path.GetFileNameWithoutExtension(romPath) + ".state");
                Spc700 spc700 = new Spc700();
                MemoryBus bus = new MemoryBus(cart, spc700);

                Cpu cpu = new Cpu(bus);
                DebugSettings.CpuVerboseLogging = false;
                DebugSettings.Spc700VerboseLogging = false;

                Renderer renderer = new Renderer();

                // The reusable debug toolchain - see Debug/IDebugTarget.cs
                // and Debug/DebugCommandProcessor.cs. Built once here so
                // both the F1 dump below and the new F4 interactive prompt
                // go through the exact same underlying data, rather than
                // each hotkey reaching into cpu/bus/ppu on its own.
                SnesDebugTarget debugTarget = new SnesDebugTarget(cpu, bus);
                DebugCommandProcessor debugCmd = new DebugCommandProcessor(debugTarget);

                // Yoshi/coin WRAM-staging investigation: registered here
                // instead of via the F4 prompt so it's active from the very
                // first CPU instruction, not from whenever a human can
                // press F4 after the window opens (hundreds of frames in).
                // Every prior trace of this range started after boot; this
                // rules out "the population happens very early and we keep
                // missing it." Remove once the investigation concludes.
                debugTarget.Watches.AddWatch("WRAM", 0x8000, 0x1800);

                int currentScanline = 0;
                long totalFrames = 0;

                while (renderer.IsOpen())
                {
                    if (currentScanline == 0)
                    {
                        bus.Interrupts.EndVBlank();
                        bus.Dma.InitHdma();
                    }

                    // Documented NMI-enable-during-vblank quirk - see
                    // PendingImmediateNmi's comment in InterruptController.cs.
                    // Checked here at the same once-per-scanline granularity
                    // as the H/V-IRQ check just below, for the same reason.
                    if (bus.Interrupts.PendingImmediateNmi)
                    {
                        bus.Interrupts.PendingImmediateNmi = false;
                        cpu.Nmi();
                    }

                    // H/V-IRQ trigger check, done here (before this scanline's CPU
                    // code runs) so a handler's register changes - e.g. SMW's classic
                    // status-bar screen split - take effect in time for THIS
                    // scanline's render, not the next one. H-only fires every line;
                    // V-only and HV fire once per frame at the target scanline. This
                    // approximates real hardware's dot-precise H-position check as
                    // "the whole target scanline", since our timing runs at
                    // per-scanline granularity rather than per-dot.
                    // Temporary escape hatch: H/V-IRQ support caused a severe
                    // regression (CPU lockup, INIDISP zeroed) - disabled by default
                    // until the actual cause is found. See conversation notes.
                    if (DebugSettings.HvIrqEnabled)
                    {
                        if (bus.Interrupts.HIrqEnabled && !bus.Interrupts.VIrqEnabled)
                        {
                            bus.Interrupts.RaiseTimerIrq();
                            cpu.Irq();
                        }
                        else if (bus.Interrupts.VIrqEnabled && currentScanline == bus.Interrupts.VTime)
                        {
                            bus.Interrupts.RaiseTimerIrq();
                            cpu.Irq();
                        }
                    }

                    int lineCycles = 0;
                    bus.LineCycles = 0;
                    while (lineCycles < CyclesPerScanline)
                    {
                        int cpuCycles = cpu.Step();
                        lineCycles += cpuCycles;
                        bus.LineCycles = lineCycles;

                        spc700.CycleBudget += cpuCycles;
                        while (spc700.CycleBudget >= 21)
                        {
                            spc700.Step();
                        }
                    }

                    bus.CurrentScanline = currentScanline;

                    if (currentScanline < 225)
                    {
                        if (currentScanline < 224) 
                        {
                            renderer.RenderScanline(bus, currentScanline);
                        }
                        bus.Dma.ExecuteHdma();
                    }

                    if (currentScanline == 225)
                    {
                        bus.Interrupts.InVBlank = true;
                        bus.Interrupts.RaiseVBlank();
                        if (bus.Interrupts.NmiEnabled) cpu.Nmi();

                        // Real hardware automatically reads the controller once per
                        // frame right at the start of vblank; mirror that timing here.
                        bus.Input.LatchAutoJoypad();
                    }

                    currentScanline++;
                    if (currentScanline >= TotalScanlines)
                    {
                        currentScanline = 0;
                        totalFrames++;
                        bus.FrameCount = totalFrames;

                        // Poll real keyboard/gamepad state once per frame and feed it
                        // into the emulated controller. This has to happen before the
                        // frame's worth of CPU steps run again (i.e. before the next
                        // LatchAutoJoypad), so doing it here - right after a frame
                        // completes - keeps the timing correct. The actual key/pad
                        // mapping lives in InputBindings.cs, not here.
                        InputBindings.ApplyInput(bus);

                        renderer.DrawFrame(bus, totalFrames);

                        // All debug/dev hotkeys (F1-F5, F9, P) live in one place -
                        // see RunHotkeys below. Previously this was ~170 lines
                        // inline here; pulled out so the actual per-scanline
                        // emulation timing loop above isn't buried under debug UI
                        // dispatch. totalFrames/currentScanline are passed by ref
                        // since F9 (load state) reassigns both from the save file.
                        RunHotkeys(cpu, bus, spc700, cart, renderer, debugTarget, debugCmd,
                            statePath, ref totalFrames, ref currentScanline);
                    }
                }
                cart.SaveSram(); // final flush on clean exit
                renderer.Shutdown();
            }
            catch (Exception ex)
            {
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
            }
        }

        // Everything triggered by a keypress or a bounded background trace,
        // checked once per completed frame. Deliberately separate from the
        // scanline-by-scanline timing loop in Main - that loop is the actual
        // emulation core and reads much more clearly without ~170 lines of
        // debug-hotkey dispatch interleaved into it. Nothing here changes
        // emulation behavior; it's read-only inspection plus the F5/F9 save-
        // state and F3 screenshot side effects, same as before the extraction.
        private static void RunHotkeys(
            Cpu cpu, MemoryBus bus, Spc700 spc700, Cartridge cart, Renderer renderer,
            SnesDebugTarget debugTarget, DebugCommandProcessor debugCmd,
            string statePath, ref long totalFrames, ref int currentScanline)
        {
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
                renderer.DumpActiveOam(bus.Ppu);
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

            // Save states (F5 save, F9 load) - one slot per ROM,
            // named to match its .srm save. This build doesn't go
            // through EmulatorSession (see that class's own header
            // comment on why the two loops are separate), so this
            // mirrors EmulatorSession.SaveState/LoadState's logic
            // directly against the local cart/cpu/bus/spc700
            // rather than sharing code with it.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F5))
            {
                try
                {
                    Directory.CreateDirectory(Path.GetDirectoryName(statePath)!);
                    using var stream = new FileStream(statePath, FileMode.Create);
                    using var w = new BinaryWriter(stream);
                    w.Write(totalFrames);
                    w.Write(currentScanline);
                    StateSerializer.Write(w, cart);
                    StateSerializer.Write(w, cpu);
                    StateSerializer.Write(w, bus);
                    StateSerializer.Write(w, spc700);
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
                    using var stream = new FileStream(statePath, FileMode.Open);
                    using var r = new BinaryReader(stream);
                    totalFrames = r.ReadInt64();
                    currentScanline = r.ReadInt32();
                    StateSerializer.Read(r, cart);
                    StateSerializer.Read(r, cpu);
                    StateSerializer.Read(r, bus);
                    StateSerializer.Read(r, spc700);
                    Console.WriteLine($"[STATE] Loaded: {statePath}");
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"[STATE] Load failed: {ex.Message}");
                }
            }
            if (_bgScrollTrace.ShouldLog())
            {
                Console.WriteLine($"[BG SCROLL] Frame {totalFrames}: BG1 X={bus.Ppu.BgScrollX[0]} Y={bus.Ppu.BgScrollY[0]}  |  BG2 X={bus.Ppu.BgScrollX[1]} Y={bus.Ppu.BgScrollY[1]}");
                if (!_bgScrollTrace.IsActive) DebugSettings.AllScrollWriteLogging = false;
            }

            if (totalFrames % StatusEveryNFrames == 0)
            {
                Console.WriteLine(
                    $"[STATUS] Frame {totalFrames} | PC=0x{cpu.PB:X2}{cpu.PC:X4} | " +
                    $"TM={bus.Ppu.Tm:X2} BGMODE={bus.Ppu.Bgmode:X2} INIDISP={bus.Ppu.Inidisp:X2}"
                );
            }

            // Periodic autosave - SRAM is a few KB at most, so this
            // is cheap even at a fairly tight interval. Protects
            // against losing a save to a crash or force-quit rather
            // than a clean exit; SaveOnExit below still covers the
            // normal case.
            if (totalFrames % SaveEveryNFrames == 0)
            {
                cart.SaveSram();
            }
        }
    }
}
