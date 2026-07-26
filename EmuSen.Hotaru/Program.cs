using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using EmuSen.Audio;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.Bindings;
using EmuSen.Serenity;

namespace EmuSen.Hotaru
{
    class Program
    {
        private static readonly DebugTools.BoundedTrace _bgScrollTrace = new();

        // Set while `feed`'s no-window mode is watching for Ctrl+C at
        // the terminal (see RunDebugPrompt's own 'feed' handling and
        // RunHotkeys' polling of it, below) - lets Ctrl+C reopen the
        // debug prompt from the terminal alone, without needing the
        // Raylib window focused to press F4. Console.TreatControlCAsInput
        // has to be true for that Ctrl+C to arrive as a readable key
        // instead of the process-terminating signal Main's own
        // PosixSignalRegistration handlers treat it as normally - reset
        // to false the moment the prompt reopens (ArmFeedWatch/
        // DisarmFeedWatch, below), by any path, so a plain Ctrl+C outside
        // 'feed' mode still means "shut down" everywhere else in this
        // frontend, the same as before 'feed' existed.
        private static bool _feedWatchActive;

        private static void ArmFeedWatch()
        {
            try { Console.TreatControlCAsInput = true; } catch { }
            _feedWatchActive = true;
        }

        private static void DisarmFeedWatch()
        {
            if (!_feedWatchActive) return;
            try { Console.TreatControlCAsInput = false; } catch { }
            _feedWatchActive = false;
        }

        // Backs the standalone shell's `core <corename> <path>` command
        // (RunStandaloneShell, below) - deliberately a small, private,
        // Hotaru-only registry rather than something shared through
        // EmuSen.DianaOS: picking which concrete ICore implementation to
        // construct for a given ROM is exactly the kind of frontend-owned
        // decision that namespace stays agnostic about on purpose (same
        // reasoning `IDebugTarget`/`ICore` themselves exist - core-
        // specific knowledge lives in the frontend or the core, never in
        // the shell). Only one entry today because only one core is
        // actually implemented (`VenusCore`, SNES) - registered under
        // both its internal codename and the console name most people
        // would actually type. Extensions gate what `core` will accept:
        // trying to load, say, a `.nes` file against `venus` is a clear
        // user error worth catching here rather than handing bytes that
        // aren't really an SNES ROM to `Cartridge`, which has no format
        // validation of its own at all (see that class's own comment on
        // copier-header stripping - it assumes SNES-shaped bytes,
        // unconditionally).
        private sealed record CoreDescriptor(string DisplayName, string[] Extensions)
        {
            public bool SupportsExtension(string extension) =>
                Array.Exists(Extensions, e => e.Equals(extension, StringComparison.OrdinalIgnoreCase));
        }

        private static readonly Dictionary<string, CoreDescriptor> _coreRegistry = new(StringComparer.OrdinalIgnoreCase)
        {
            ["venus"] = new CoreDescriptor("SNES (Venus)", new[] { ".smc", ".sfc" }),
            ["snes"] = new CoreDescriptor("SNES (Venus)", new[] { ".smc", ".sfc" }),
        };

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
            // DianaOS is now the ONLY thing shown at launch, whether or
            // not a ROM path was given on the command line - a CLI arg is
            // just a shortcut for typing 'core <name> <path>' at the
            // shell prompt yourself, not a separate way to skip the shell
            // entirely. RunStandaloneShell prints the shell banner either
            // way and, given a starting path, either returns it
            // immediately (valid file - go straight to gameplay) or
            // reports why not and falls through into the same interactive
            // loop a bare launch gets, so a typo'd CLI arg doesn't just
            // dead-end the process. Returns null only if the user actually
            // asked to shut down (or hit EOF) with no ROM ever resolved,
            // in which case there's nothing left to do.
            string? romPathFromShell = RunStandaloneShell(args.Length > 0 ? args[0] : null);
            if (romPathFromShell is null) return;
            string romPath = romPathFromShell;

            try
            {
                // Separate from Saves/ (battery-backed cartridge SRAM,
                // owned by Cartridge.SavePath) - a save state is a full
                // snapshot of emulator state, a different kind of artifact
                // with a different lifetime (SRAM persists across sessions
                // like real hardware; a save state is a debugging/
                // convenience tool), so the two shouldn't land in the same
                // folder just because they're both "saves".
                string statePath = Path.Combine(Directory.GetCurrentDirectory(), "SaveStates", Path.GetFileNameWithoutExtension(romPath) + ".state");

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
                // The frame-timings delegate feeds `coretop`'s hardware-load
                // bars (IDebugTarget.GetHardwareLoad) - see SnesDebugTarget's
                // own constructor comment for why this can't just read them
                // off Cpu/Bus/Renderer directly.
                SnesDebugTarget debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!,
                    () => (core.LastFrameCpuSpc700Ms, core.LastFramePpuMs, core.LastFrameHdmaMs));
                // Wires `coretop -w` up to a real window (AvaloniaHost) -
                // replaces the standard registry's plain CoretopCommand
                // (no window support) via extraCommands' override-by-name
                // behavior, same mechanism EmuSen.Mistress9 uses to
                // replace `coretop` outright.
                DianaOSInterpreter debugCmd = DianaOSInterpreter.CreateDefault(debugTarget,
                    new IDianaOSCommand[] { new EmuSen.DianaOS.Commands.CoretopCommand(AvaloniaHost.ShowCoretopWindow) });

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

                while (presenter.IsOpen())
                {
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
                        if (RunDebugPrompt(core, debugTarget, debugCmd, statePath)) break; // 'shutdown' typed - fall through to the same clean shutdown as closing the window
                        continue;
                    }

                    PumpAudio(core, audioStream);

                    // Must happen before the next RunFrame()'s own
                    // LatchAutoJoypad - see EmuSen_Frontend_Driver.md §1.
                    InputBindings.ApplyInput(core.Bus!);

                    presenter.Present(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight,
                        () => core.Renderer!.DrawDebugPanels(core.Bus!, core.TotalFrames));

                    // All debug/dev hotkeys - see EmuSen_Frontend_Driver.md §2.
                    if (RunHotkeys(core, presenter, debugTarget, debugCmd, frameRecorder, statePath)) break; // 'shutdown' typed at the F4 prompt
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

        // Drains whatever's queued into the core's own audio buffer into
        // the actual output device, once per RunFrame() call. Only pulls
        // when Raylib says its internal buffer is ready for more
        // (IsAudioStreamProcessed) - pushing data it hasn't asked for yet
        // isn't how raylib's streaming API is meant to be driven. Capped
        // at 4096 frames per call so a stall (e.g. time spent in the F4
        // debug prompt) that lets the queue build up doesn't dump an
        // enormous, laggy chunk in one call - the core's own buffer cap
        // (AudioSettings.AudioBufferMaxSamples) already discards the
        // oldest samples once that fills, so the rest is simply left
        // queued for the next call(s) rather than lost.
        //
        // Goes through ICore.DequeueAudioSamples() rather than reaching
        // into core.Spc700.Dsp.AudioBuffer directly - the same
        // core-agnostic split GetFrameBufferRgba() already established for
        // video (see FramePresenter's own comment), closed here too during
        // a pass adding real audio output to EmuSen.Mistress9, which
        // needed this exact same drain logic and had no Venus-specific
        // access of its own to duplicate it against.
        private static unsafe void PumpAudio(ICore core, Raylib_cs.AudioStream stream)
        {
            if (!Raylib_cs.Raylib.IsAudioStreamProcessed(stream)) return;

            short[] data = core.DequeueAudioSamples(4096);
            if (data.Length == 0) return;

            fixed (short* p = data)
            {
                Raylib_cs.Raylib.UpdateAudioStream(stream, p, data.Length / 2);
            }
        }

        // Hotaru's entire launch experience now: no core, no window, no
        // audio device yet, just DianaOS against a null IDebugTarget -
        // the shell is the first and only thing shown, whether Hotaru was
        // launched bare or with a ROM path on the command line. Every
        // general-purpose command (echo/sed/grep/awk/ls/cd/source/if/
        // for/while/...) works exactly as it would with a ROM loaded;
        // anything needing real hardware access (mem/regs/watch/...)
        // reports a clean "No ROM loaded" instead of erroring, the same
        // graceful-degradation every command's own RequireTarget guard
        // already provides. Deliberately its own small loop rather than
        // reusing RunDebugPrompt with a null core - that method's
        // 'step'/'state' shortcuts and its "resume the game" framing
        // don't mean anything here, and threading null-core checks
        // through it would make the actually-in-a-game case harder to
        // follow for no real benefit.
        //
        // `initialRomPath` is whatever came in as a CLI arg, if anything -
        // treated purely as a shortcut for typing 'core <name> <path>'
        // yourself, not a separate launch path that bypasses the shell: a
        // valid file resolves and returns immediately (no interactive
        // loop needed), an invalid one just reports why and falls through
        // into the same loop a bare launch gets, so a typo'd CLI arg
        // still lands you at a usable prompt instead of a dead process.
        //
        // Returns the ROM path to actually launch (once `core` - or the
        // initial arg - resolves one), or null if the user asked to shut
        // down instead (or hit EOF) - `Main` treats null as "nothing more
        // to do" and returns. `core` is handled here, not as a
        // DianaOSInterpreter command, for the same reason 'resume'/
        // 'shutdown' are: it needs to hand control back to THIS loop's
        // caller (to actually go build a VenusCore/window/audio device
        // and start running), which a command's own "return a string"
        // contract has no way to do.
        private static string? RunStandaloneShell(string? initialRomPath)
        {
            DianaOSInterpreter shell = DianaOSInterpreter.CreateDefault(null);
            Console.WriteLine("--- DianaOS (type 'help', 'core <name> <path>' to launch a game, 'shutdown' to quit) ---");

            if (initialRomPath != null)
            {
                if (File.Exists(initialRomPath))
                {
                    Console.WriteLine($"[ROM] Loading: {initialRomPath}");
                    return initialRomPath;
                }
                Console.WriteLine($"[ERROR] ROM not found: {initialRomPath}");
            }

            while (true)
            {
                Console.Write(shell.IsAwaitingMoreInput ? "> " : "DianaOS #: ");
                string? line = ConsoleLineReader.ReadLine(shell.History.Entries);
                if (line is null) return null;
                string trimmed = line.Trim();

                // Same 'shutdown'/'quit' keywords RunDebugPrompt recognizes -
                // see that method's own comment on why this is 'shutdown',
                // not 'exit', and why both loops use the exact same word
                // for "actually terminate the process" now. There's no
                // 'resume' here (nothing to resume without a game running),
                // so unlike RunDebugPrompt this loop only ever has the one
                // way out besides EOF.
                if (!shell.IsAwaitingMoreInput
                    && (trimmed.Equals("shutdown", StringComparison.OrdinalIgnoreCase) || trimmed.Equals("quit", StringComparison.OrdinalIgnoreCase)))
                {
                    return null;
                }

                if (!shell.IsAwaitingMoreInput && trimmed.StartsWith("core ", StringComparison.OrdinalIgnoreCase))
                {
                    string? validatedRomPath = TryResolveCoreCommand(trimmed);
                    if (validatedRomPath != null) return validatedRomPath;
                    continue; // TryResolveCoreCommand already printed why it failed
                }

                string output = shell.Execute(line);
                if (output.Length > 0) Console.WriteLine(output);
            }
        }

        // `core <corename> <path>` - not a DianaOSInterpreter command
        // (see RunStandaloneShell's own comment); handled as plain string
        // parsing here instead, the same way 'state save|load' is inside
        // RunDebugPrompt. Prints its own error and returns null for
        // anything wrong (usage, unknown core, missing file, unsupported
        // extension) so the caller can just loop back to the prompt -
        // only a fully validated ROM path is ever returned.
        private static string? TryResolveCoreCommand(string trimmedLine)
        {
            string[] parts = trimmedLine.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length != 3)
            {
                Console.WriteLine("Usage: core <corename> <path-to-rom>");
                return null;
            }

            string coreName = parts[1];
            string romPath = parts[2];

            if (!_coreRegistry.TryGetValue(coreName, out CoreDescriptor? descriptor))
            {
                Console.WriteLine($"core: unknown core '{coreName}'. Supported: {string.Join(", ", new SortedSet<string>(_coreRegistry.Keys, StringComparer.OrdinalIgnoreCase))}");
                return null;
            }

            if (!File.Exists(romPath))
            {
                Console.WriteLine($"core: ROM not found: {romPath}");
                return null;
            }

            string extension = Path.GetExtension(romPath);
            if (!descriptor.SupportsExtension(extension))
            {
                Console.WriteLine($"core: '{(extension.Length > 0 ? extension : "(no extension)")}' is not a supported ROM type for {descriptor.DisplayName} - expected: {string.Join(", ", descriptor.Extensions)}");
                return null;
            }

            return romPath;
        }

        // Shared by the F4 hotkey and the main loop's own breakpoint-halt
        // check, so both funnel through the exact same interactive
        // command loop rather than keeping two copies in sync (the same
        // reasoning VenusCore's own header comment gives for why RunFrame()
        // itself isn't duplicated between frontends).
        //
        // 'resume'/'continue'/'c' and 'shutdown'/'quit' are handled here,
        // not registered as DianaOSInterpreter commands, because they need
        // to make this loop return control to the OUTER per-frame loop so
        // it can actually call core.RunFrame() again (to resume) or break
        // its own while loop (to shut down) - a DianaOSInterpreter command
        // only ever returns a string to print, it has no way to affect
        // control flow one level up. Returns true if the caller should
        // shut down the whole process instead of resuming the game -
        // 'shutdown' is the SAME word RunStandaloneShell recognizes for
        // the same "actually quit" meaning, so it means one consistent
        // thing everywhere in this frontend, unlike the old 'exit' (which
        // used to mean "resume the game" here but "quit the process" in
        // RunStandaloneShell - the same word, two different actions,
        // exactly the inconsistency 'resume'/'shutdown' now removes).
        private static bool RunDebugPrompt(VenusCore core, SnesDebugTarget debugTarget, DianaOSInterpreter debugCmd, string statePath)
        {
            // Whatever got us back into this prompt - F4, a breakpoint
            // halt, or 'feed's own Ctrl+C watch below reopening it - none
            // of them should leave Ctrl+C meaning anything other than
            // "shut down" once we're actually sitting at a prompt again.
            DisarmFeedWatch();

            Console.WriteLine("--- DianaOS (type 'help', 'resume' to resume, 'feed'/'feed -w' to resume and watch gameplay, 'shutdown' to quit, 'step'/'s' to single-step) ---");
            while (true)
            {
                // Bash-style secondary prompt while a quote/"$(...)"/
                // if-else-fi/for-do-done block is still open (see
                // DianaOSInterpreter.IsAwaitingMoreInput's own comment) -
                // and, just as importantly, every one of the single-word
                // REPL shortcuts below (resume/continue/c/shutdown/quit/
                // step/s/"state ...") is suppressed while awaiting more
                // input, so typing the shell's own `continue`/`break`
                // keywords (or any other word that happens to collide with
                // one of these) while composing a loop body reaches the
                // shell instead of being hijacked as "resume emulation" -
                // the same way a real bash prompt never mistakes a
                // `continue` typed inside an unfinished `if` for the
                // reader's own control commands.
                Console.Write(debugCmd.IsAwaitingMoreInput ? "> " : "DianaOS #: ");
                string? line = ConsoleLineReader.ReadLine(debugCmd.History.Entries);
                if (line is null) return false;
                string trimmed = line.Trim();

                if (!debugCmd.IsAwaitingMoreInput)
                {
                    if (trimmed.Length == 0
                        || trimmed.Equals("resume", StringComparison.OrdinalIgnoreCase)
                        || trimmed.Equals("continue", StringComparison.OrdinalIgnoreCase)
                        || trimmed.Equals("c", StringComparison.OrdinalIgnoreCase))
                    {
                        break;
                    }
                    if (trimmed.Equals("shutdown", StringComparison.OrdinalIgnoreCase) || trimmed.Equals("quit", StringComparison.OrdinalIgnoreCase))
                    {
                        return true;
                    }
                    // 'feed'/'feed -w' - like 'resume' (below), but also
                    // arms a way back into this prompt that doesn't need
                    // the Raylib window focused: Ctrl+C at the terminal
                    // (RunHotkeys polls for it every frame while armed -
                    // see ArmFeedWatch's own comment). '-w' additionally
                    // opens a live-mirrored Avalonia window
                    // (AvaloniaHost.ShowFeedWindow) showing the actual
                    // game picture, not hardware/debug data the way
                    // `coretop -w` does - same reasoning as that command
                    // for why this needs its own window/thread at all
                    // (Raylib supports exactly one native window).
                    if (trimmed.Equals("feed", StringComparison.OrdinalIgnoreCase) || trimmed.StartsWith("feed ", StringComparison.OrdinalIgnoreCase))
                    {
                        string[] feedParts = trimmed.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                        bool windowed = feedParts.Length >= 2 && feedParts[1].Equals("-w", StringComparison.OrdinalIgnoreCase);
                        if (windowed)
                        {
                            AvaloniaHost.ShowFeedWindow(() => (core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight));
                            Console.WriteLine("[FEED] Opened in a separate window.");
                        }
                        ArmFeedWatch();
                        Console.WriteLine("Resuming - press Ctrl+C in this terminal to reopen the prompt.");
                        break;
                    }
                }
                // Text-command equivalent of the F5/F9 hotkeys - added
                // specifically because F5/F9 depend on the Raylib window
                // having keyboard focus, which is easy to lose track of,
                // while this prompt (already reliably reachable via F4)
                // gives an unambiguous confirmation line either way. Same
                // statePath both hotkeys use, so a state made one way loads
                // fine the other.
                if (!debugCmd.IsAwaitingMoreInput && trimmed.StartsWith("state ", StringComparison.OrdinalIgnoreCase))
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
                if (!debugCmd.IsAwaitingMoreInput && (trimmed.Equals("step", StringComparison.OrdinalIgnoreCase) || trimmed.Equals("s", StringComparison.OrdinalIgnoreCase)))
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
            return false;
        }

        // Every debug/dev hotkey - see EmuSen_Frontend_Driver.md §2. Returns
        // true if the F4 prompt was used to request a shutdown (see
        // RunDebugPrompt's own comment) - the caller (Main's own per-frame
        // loop) breaks out and falls through to the same clean shutdown
        // sequence closing the window normally takes.
        private static bool RunHotkeys(
            VenusCore core, FramePresenter presenter, SnesDebugTarget debugTarget, DianaOSInterpreter debugCmd, FrameRecorder frameRecorder,
            string statePath)
        {
            // 'feed's own way back into the prompt - see ArmFeedWatch's
            // own comment. A cheap non-blocking poll (Console.KeyAvailable),
            // same technique `coretop`'s own dashboard uses for Ctrl+C,
            // just spread across frames here instead of a dedicated
            // blocking loop, since this has to coexist with gameplay
            // actually running rather than pausing it. Guarded on
            // IsInputRedirected the same way coretop's own Execute is -
            // KeyAvailable throws if there's no real console to poll.
            if (_feedWatchActive && !Console.IsInputRedirected && Console.KeyAvailable)
            {
                ConsoleKeyInfo key = Console.ReadKey(intercept: true);
                if ((key.Modifiers & ConsoleModifiers.Control) != 0 && key.Key == ConsoleKey.C)
                {
                    Console.WriteLine();
                    if (RunDebugPrompt(core, debugTarget, debugCmd, statePath)) return true; // 'shutdown' typed after reopening via 'feed's Ctrl+C
                }
            }

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

            // Compatibility toggle for games that read Controller 2 instead
            // of Controller 1 (Super Mario All-Stars' classic sub-games
            // being the known case - see InputBindings.MirrorPlayer1ToPlayer2's
            // own comment). Off by default; leave off for real two-player
            // sessions.
            if (Raylib_cs.Raylib.IsKeyPressed(Raylib_cs.KeyboardKey.F7))
            {
                InputBindings.MirrorPlayer1ToPlayer2 = !InputBindings.MirrorPlayer1ToPlayer2;
                Console.WriteLine($"[INPUT] Mirror Player 1 -> Player 2: {(InputBindings.MirrorPlayer1ToPlayer2 ? "ON" : "OFF")}");
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
                if (RunDebugPrompt(core, debugTarget, debugCmd, statePath)) return true; // 'shutdown' typed
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

            // Periodic SRAM autosave now happens inside VenusCore.RunFrame()
            // itself (see that class) - no longer duplicated here.
            return false;
        }
    }
}
