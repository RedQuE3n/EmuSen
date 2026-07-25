using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using EmuSen.Audio;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Debug;
using EmuSen.Bindings;
using EmuSen.Presentation;

namespace EmuSen.RaylibFrontend
{
    class Program
    {
        private const int StatusEveryNFrames = 60;

        private static readonly DebugTools.BoundedTrace _bgScrollTrace = new();

        static void Main(string[] args)
        {
            TextWriter originalOut = Console.Out;

            // Disposed by FlushAndDispose() below on every exit path
            // (normal completion, the catch block's own CPU-HALT print,
            // Ctrl+C, `kill <pid>`, or an unhandled exception on some
            // other thread) - CategorizedLogWriter's file writes now
            // happen on a background thread with no AutoFlush, so
            // something has to explicitly wait for that queue to drain
            // before the process exits, or the last batch of buffered log
            // lines is silently lost. The old synchronous, AutoFlush=true
            // design never needed this; this is the correctness cost of
            // moving file I/O off the emulation thread.
            CategorizedLogWriter? logWriter = null;

            // Needed by FlushAndDispose to flush Cpu/Spc700's
            // RepeatCollapsingTrace state before the log writer closes -
            // see that call's own comment below.
            VenusCore? core = null;

            // Guards FlushAndDispose against running twice - Ctrl+C alone
            // can reach it via both the SIGINT registration below and the
            // ProcessExit that follows once the default handler decides to
            // terminate, and a crash can reach it via both
            // UnhandledException and the finally block. Dispose() itself
            // isn't safe to call twice (CategorizedLogWriter closes file
            // handles the second call would touch again), so only the
            // first caller should actually run it.
            int shutdownGuard = 0;

            void FlushAndDispose()
            {
                if (Interlocked.Exchange(ref shutdownGuard, 1) != 0) return;

                // Must run before logWriter.Dispose() below, while
                // Console.Out is still routed through it - otherwise a
                // loop CpuVerboseLogging/Spc700VerboseLogging was still
                // mid-repeat on would never reach the file at all. See
                // DebugTools.RepeatCollapsingTrace<TKey>.Flush().
                core?.Cpu?.FlushVerboseTrace();
                core?.Spc700?.FlushVerboseTrace();

                Console.SetOut(originalOut);
                logWriter?.Dispose();
            }

            // Covers every exit path the try/finally below can't reach on
            // its own: `kill <pid>` (SIGTERM) or Ctrl+C (SIGINT) from a
            // terminal, and an unhandled exception thrown on a thread
            // other than this one (e.g. inside Raylib's native callbacks).
            // None of those unwind through Main's own try/finally, so
            // without this, whatever CategorizedLogWriter still had
            // queued would be silently lost - exactly the kind of
            // ungraceful exit that produced the empty/truncated log files
            // diagnosed earlier. Cannot catch a hard `kill -9`/SIGKILL -
            // nothing in userspace can intercept that signal at all.
            AppDomain.CurrentDomain.ProcessExit += (_, _) => FlushAndDispose();
            AppDomain.CurrentDomain.UnhandledException += (_, _) => FlushAndDispose();
            using PosixSignalRegistration sigTerm = PosixSignalRegistration.Create(PosixSignal.SIGTERM, ctx =>
            {
                FlushAndDispose();
                ctx.Cancel = false; // let the process actually terminate after flushing
            });
            using PosixSignalRegistration sigInt = PosixSignalRegistration.Create(PosixSignal.SIGINT, ctx =>
            {
                FlushAndDispose();
                ctx.Cancel = false;
            });

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
                core = new VenusCore(headless: false);

                // Log dir setup - deliberately placed here, not earlier. See
                // EmuSen_Frontend_Driver.md §1.
                string logDir = Path.Combine(Directory.GetCurrentDirectory(), "Logs", core.CoreName, $"console_{DateTime.Now:yyyyMMdd_HHmmss}");
                Directory.CreateDirectory(logDir);
                logWriter = new CategorizedLogWriter(originalOut, logDir);
                Console.SetOut(logWriter);

                core.LoadRom(romPath);
                // On by default for the current test pass - previously
                // forced off right after load, which is why cpu.log/apu.log
                // only ever had the one-time startup lines.
                DebugSettings.CpuVerboseLogging = true;
                DebugSettings.Spc700VerboseLogging = true;

                // Debug toolchain, built once - see EmuSen_Frontend_Driver.md §1.
                SnesDebugTarget debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!);
                DebugCommandProcessor debugCmd = new DebugCommandProcessor(debugTarget);

                FrameRecorder frameRecorder = new FrameRecorder(debugTarget);

                // Power-on watch registration (Yoshi/coin investigation) -
                // see EmuSen_Frontend_Driver.md §1.
                debugTarget.Watches.AddWatch("WRAM", 0x8000, 0x1800);
                debugTarget.Watches.AddWatch("WRAM", 0x0D80, 0x0080);

                // FramePresenter owns the window and drives every core
                // through the agnostic ICore.GetFrameBufferRgba() contract
                // instead of Renderer.DrawFrame reaching into Venus-specific
                // internals directly - see FramePresenter's own comment.
                // Renderer.DrawDebugPanels still needs bus.Ppu for its
                // VRAM/CGRAM/register overlays, which is exactly the kind
                // of concrete-core debug access ICore.cs says is expected
                // to stay off the agnostic interface.
                using FramePresenter presenter = new FramePresenter();

                // Audio output - see EmuSen_Frontend_Driver.md §1. SDsp
                // already generates correct samples into
                // core.Spc700.Dsp.AudioBuffer at AudioSettings.SampleRate
                // (32kHz stereo, interleaved L/R shorts) - this is the only
                // piece that was missing: draining that queue into a real
                // output device. Buffer size chosen as a middle ground
                // between latency (smaller = less audible lag behind the
                // picture) and safety margin against an occasional slow
                // frame starving the stream into an audible glitch/pop.
                // Must be >= PumpAudio's own per-call cap (4096 frames,
                // used to drain a backlog after a stall like the F4 debug
                // prompt) - a smaller buffer than that made every catch-up
                // push exceed the stream's actual capacity, which is what
                // raylib's "Attempting to write too many frames to buffer"
                // warning was reporting on every such call.
                Raylib_cs.Raylib.SetAudioStreamBufferSizeDefault(4096);
                Raylib_cs.Raylib.InitAudioDevice();
                Raylib_cs.AudioStream audioStream = Raylib_cs.Raylib.LoadAudioStream((uint)AudioSettings.SampleRate, 16, 2);
                Raylib_cs.Raylib.PlayAudioStream(audioStream);

                // Same measured-fps diagnostic Testing Studio's status bar
                // has (EmuSen.TestingStudio/Views/MainWindow.axaml.cs), added
                // here to answer the same question for this build: does
                // RunFrame() itself slow down under real gameplay the same
                // way it does there, or is that slowdown specific to the
                // Avalonia frontend? Since RunFrame() is identical shared
                // code (VenusCore.cs), the expectation is this build pays
                // the same cost - just never visible before since nothing
                // here measured real wall-clock fps.
                Stopwatch fpsClock = Stopwatch.StartNew();
                TimeSpan fpsWindowStart = fpsClock.Elapsed;
                int fpsFramesInWindow = 0;
                TimeSpan runFrameTimeInWindow = TimeSpan.Zero;
                TimeSpan totalTimeInWindow = TimeSpan.Zero;
                Stopwatch frameStopwatch = new Stopwatch();

                // VenusCore's own per-scanline-granularity phase breakdown
                // (VenusCore.RunFrame()'s own comment) - which subsystem
                // (CPU+SPC700 stepping, PPU rendering, HDMA) actually
                // accounts for RunFrame() getting slower during gameplay.
                double cpuSpc700MsInWindow = 0;
                double ppuMsInWindow = 0;
                double hdmaMsInWindow = 0;
                double objEvalMsInWindow = 0;
                double blendMsInWindow = 0;
                double mainCompositeMsInWindow = 0;
                double subCompositeMsInWindow = 0;

                while (presenter.IsOpen())
                {
                    TimeSpan iterationStart = fpsClock.Elapsed;

                    frameStopwatch.Restart();
                    core.RunFrame();

                    // A breakpoint (or an armed single-step - see
                    // BreakpointRegistry) halted RunFrame() before it
                    // finished this frame - the frame buffer is only
                    // partially rendered, so skip presenting/hotkeys this
                    // iteration entirely and go straight to the same prompt
                    // F4 already uses. Looping back to the top afterward
                    // calls RunFrame() again, which resumes exactly where
                    // it left off (see that method's own comment).
                    if (core.IsHaltedAtBreakpoint)
                    {
                        Console.WriteLine($"\n[BREAKPOINT] Halted at ${core.HaltedAddress:X6}");
                        Console.WriteLine(debugTarget.GetSummaryText());
                        RunDebugPrompt(core, debugTarget, debugCmd, statePath);
                        continue;
                    }

                    PumpAudio(core, audioStream);

                    TimeSpan runFrameElapsed = frameStopwatch.Elapsed;
                    cpuSpc700MsInWindow += core.LastFrameCpuSpc700Ms;
                    ppuMsInWindow += core.LastFramePpuMs;
                    hdmaMsInWindow += core.LastFrameHdmaMs;
                    objEvalMsInWindow += core.LastFrameObjEvalMs;
                    blendMsInWindow += core.LastFrameBlendMs;
                    mainCompositeMsInWindow += core.LastFrameMainCompositeMs;
                    subCompositeMsInWindow += core.LastFrameSubCompositeMs;

                    // Must happen before the next RunFrame()'s own
                    // LatchAutoJoypad - see EmuSen_Frontend_Driver.md §1.
                    InputBindings.ApplyInput(core.Bus!);

                    presenter.Present(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight,
                        () => core.Renderer!.DrawDebugPanels(core.Bus!, core.TotalFrames));

                    // All debug/dev hotkeys - see EmuSen_Frontend_Driver.md §2.
                    RunHotkeys(core, presenter, debugTarget, debugCmd, frameRecorder, statePath);

                    fpsFramesInWindow++;
                    runFrameTimeInWindow += runFrameElapsed;
                    totalTimeInWindow += fpsClock.Elapsed - iterationStart;

                    TimeSpan fpsWindowElapsed = fpsClock.Elapsed - fpsWindowStart;
                    if (fpsWindowElapsed >= TimeSpan.FromSeconds(1))
                    {
                        double fps = fpsFramesInWindow / fpsWindowElapsed.TotalSeconds;
                        double runMs = runFrameTimeInWindow.TotalMilliseconds / fpsFramesInWindow;
                        double totalMs = totalTimeInWindow.TotalMilliseconds / fpsFramesInWindow;
                        double cpuSpc700Ms = cpuSpc700MsInWindow / fpsFramesInWindow;
                        double ppuMs = ppuMsInWindow / fpsFramesInWindow;
                        double hdmaMs = hdmaMsInWindow / fpsFramesInWindow;
                        double objEvalMs = objEvalMsInWindow / fpsFramesInWindow;
                        double blendMs = blendMsInWindow / fpsFramesInWindow;
                        double mainCompositeMs = mainCompositeMsInWindow / fpsFramesInWindow;
                        double subCompositeMs = subCompositeMsInWindow / fpsFramesInWindow;
                        Console.WriteLine(
                            $"[FPS] {fps:F1} fps (run {runMs:F2}ms / total {totalMs:F2}ms) " +
                            $"[cpu+apu {cpuSpc700Ms:F2}ms / ppu {ppuMs:F2}ms / hdma {hdmaMs:F2}ms] " +
                            $"(ppu breakdown: objEval {objEvalMs:F2}ms / main {mainCompositeMs:F2}ms / " +
                            $"sub {subCompositeMs:F2}ms / blend {blendMs:F2}ms)");
                        fpsFramesInWindow = 0;
                        runFrameTimeInWindow = TimeSpan.Zero;
                        totalTimeInWindow = TimeSpan.Zero;
                        cpuSpc700MsInWindow = 0;
                        ppuMsInWindow = 0;
                        hdmaMsInWindow = 0;
                        objEvalMsInWindow = 0;
                        blendMsInWindow = 0;
                        mainCompositeMsInWindow = 0;
                        subCompositeMsInWindow = 0;
                        fpsWindowStart = fpsClock.Elapsed;
                    }
                }
                core.SaveSram(); // final flush on clean exit
                Raylib_cs.Raylib.UnloadAudioStream(audioStream);
                Raylib_cs.Raylib.CloseAudioDevice();
                presenter.Shutdown();
            }
            catch (Exception ex)
            {
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
            }
            finally
            {
                // Normal completion and the catch block's own CPU-HALT
                // print both funnel through here; the signal/ProcessExit
                // handlers above cover the exit paths that never reach
                // this finally at all. FlushAndDispose's own guard makes
                // it safe if one of those already ran first.
                FlushAndDispose();
            }
        }

        // Drains whatever SDsp has queued into core.Spc700.Dsp.AudioBuffer
        // (interleaved L/R shorts at AudioSettings.SampleRate - see that
        // class's own comment) into the actual output device, once per
        // RunFrame() call. Only pulls when Raylib says its internal buffer
        // is ready for more (IsAudioStreamProcessed) - pushing data it
        // hasn't asked for yet isn't how raylib's streaming API is meant to
        // be driven. Capped at 4096 frames per call so a stall (e.g. time
        // spent in the F4 debug prompt) that lets the queue build up
        // doesn't dump an enormous, laggy chunk in one call - AudioBuffer's
        // own 1-second cap (AudioSettings.AudioBufferMaxSamples) already
        // discards the oldest samples once that fills, so the rest is
        // simply left queued for the next call(s) rather than lost.
        private static unsafe void PumpAudio(VenusCore core, Raylib_cs.AudioStream stream)
        {
            if (!Raylib_cs.Raylib.IsAudioStreamProcessed(stream)) return;

            var buffer = core.Spc700!.Dsp.AudioBuffer;
            int framesAvailable = buffer.Count / 2; // interleaved L/R
            if (framesAvailable == 0) return;

            int framesToSend = Math.Min(framesAvailable, 4096);
            short[] data = new short[framesToSend * 2];
            for (int i = 0; i < data.Length; i++) data[i] = buffer.Dequeue();

            fixed (short* p = data)
            {
                Raylib_cs.Raylib.UpdateAudioStream(stream, p, framesToSend);
            }
        }

        // Shared by the F4 hotkey and the main loop's own breakpoint-halt
        // check, so both funnel through the exact same interactive
        // command loop rather than keeping two copies in sync (the same
        // reasoning VenusCore's own header comment gives for why RunFrame()
        // itself isn't duplicated between frontends).
        //
        // 'step'/'s' and 'continue'/'c' are handled here, not registered as
        // DebugCommandProcessor commands, because they need to make this
        // loop return control to the OUTER per-frame loop so it can
        // actually call core.RunFrame() again - a DebugCommandProcessor
        // command only ever returns a string to print, it has no way to
        // affect control flow one level up. 'exit'/'quit' already worked
        // this same way before breakpoints existed at all.
        private static void RunDebugPrompt(VenusCore core, SnesDebugTarget debugTarget, DebugCommandProcessor debugCmd, string statePath)
        {
            Console.WriteLine("--- Debug prompt (type 'help', 'exit' to resume, 'step'/'s' to single-step) ---");
            while (true)
            {
                Console.Write("debug> ");
                string? line = Console.ReadLine();
                if (line is null) break;
                string trimmed = line.Trim();
                if (trimmed.Length == 0
                    || trimmed.Equals("exit", StringComparison.OrdinalIgnoreCase)
                    || trimmed.Equals("quit", StringComparison.OrdinalIgnoreCase)
                    || trimmed.Equals("continue", StringComparison.OrdinalIgnoreCase)
                    || trimmed.Equals("c", StringComparison.OrdinalIgnoreCase))
                {
                    break;
                }
                // Text-command equivalent of the F5/F9 hotkeys - added
                // specifically because F5/F9 depend on the Raylib window
                // having keyboard focus, which is easy to lose track of,
                // while this prompt (already reliably reachable via F4)
                // gives an unambiguous confirmation line either way. Same
                // statePath both hotkeys use, so a state made one way loads
                // fine the other.
                if (trimmed.StartsWith("state ", StringComparison.OrdinalIgnoreCase))
                {
                    string[] stateParts = trimmed.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                    string sub = stateParts.Length >= 2 ? stateParts[1].ToLowerInvariant() : "";
                    string path = stateParts.Length >= 3 ? stateParts[2] : statePath;
                    if (sub == "save")
                    {
                        try
                        {
                            core.SaveState(path);
                            Console.WriteLine($"[STATE] Saved: {path}");
                        }
                        catch (Exception ex)
                        {
                            Console.WriteLine($"[STATE] Save failed: {ex.Message}");
                        }
                    }
                    else if (sub == "load")
                    {
                        try
                        {
                            core.LoadState(path);
                            Console.WriteLine($"[STATE] Loaded: {path}");
                        }
                        catch (Exception ex)
                        {
                            Console.WriteLine($"[STATE] Load failed: {ex.Message}");
                        }
                    }
                    else
                    {
                        Console.WriteLine("Usage: state save|load [path]  (defaults to the F5/F9 path if omitted)");
                    }
                    continue;
                }
                if (trimmed.Equals("step", StringComparison.OrdinalIgnoreCase) || trimmed.Equals("s", StringComparison.OrdinalIgnoreCase))
                {
                    // Arms a one-shot halt-before-next-instruction (see
                    // BreakpointRegistry.ArmSingleStep) and hands control
                    // back to the outer loop, which is the only thing that
                    // can actually call RunFrame() to let that instruction
                    // run. If we're resuming from an existing breakpoint
                    // halt, RunFrame() executes exactly the halted-at
                    // instruction before re-checking, so this correctly
                    // steps ONE instruction forward either way - see
                    // RunFrame()'s own _justResumedFromBreakpoint comment.
                    debugTarget.Breakpoints.ArmSingleStep();
                    Console.WriteLine("Stepping one instruction...");
                    break;
                }
                Console.WriteLine(debugCmd.Execute(trimmed));
            }
            Console.WriteLine("--- Resuming ---");
        }

        // Every debug/dev hotkey - see EmuSen_Frontend_Driver.md §2.
        private static void RunHotkeys(
            VenusCore core, FramePresenter presenter, SnesDebugTarget debugTarget, DebugCommandProcessor debugCmd, FrameRecorder frameRecorder,
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

            // Full backdrop/window/OAM debug dump - moved here from the old
            // Renderer.DrawFrame (now gone, see FramePresenter) since it's
            // pure Console logging with no render-target dependency.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.O))
            {
                core.Renderer!.DumpBackdropAndWindowDebugInfo(core.Bus!.Ppu, core.TotalFrames);
            }

            // Cycles the prototype post-processing shader pass (None ->
            // Scanlines -> Crt -> None) - manual A/B testing until there's
            // a real settings UI for this. See FramePresenter/ShaderEffect.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F8))
            {
                ShaderEffect effect = presenter.CycleEffect();
                Console.WriteLine($"[SHADER] Active effect: {effect}");
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
                RunDebugPrompt(core, debugTarget, debugCmd, statePath);
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
