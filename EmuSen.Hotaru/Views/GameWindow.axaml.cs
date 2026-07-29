using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Threading;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Graphics;
using EmuSen.Hotaru.Audio;
using EmuSen.Hotaru.Imaging;
using EmuSen.Hotaru.Input;
using EmuSen.Serenity;

namespace EmuSen.Hotaru.Views
{
    // Hotaru's game window - owns the actual native Window (keyboard
    // capture, Closing sequence) plus a GameFrameControl (from
    // EmuSen.Serenity) as its content, and starts/stops the background
    // emulation thread. Deliberately does NOT go through
    // EmuSen.Serenity.FramePresenter, even though that class exists
    // specifically to bundle a Window + GameFrameControl together -
    // FramePresenter's whole point is "just the game, no chrome" for a
    // consumer that needs nothing else from its window, but this window
    // needs real keyboard capture, a Closing handler, and to host every
    // hotkey/debug-prompt concern below, none of which FramePresenter's
    // generic contract exposes. Mistress's own MainWindow makes the same
    // choice (it inlines its own coalescing present logic against a
    // WriteableBitmap rather than going through any shared presenter
    // class) - this follows that same established precedent, just against
    // GameFrameControl instead of a WriteableBitmap. FramePresenter stays
    // available, untouched, for a future consumer that only needs "a
    // window with the game in it" and nothing more (see its own header
    // comment on why it exists at all).
    //
    // Threading model - see this project's own migration plan: the
    // background emulation thread (EmulationLoop, started in this
    // constructor) owns RunFrame(), audio, hotkey dispatch, and the
    // blocking F4 DianaOS console prompt itself; the Avalonia UI thread
    // (this class's event handlers) owns the window, keyboard capture,
    // and gamepad polling. No pause/resume mechanism is needed for
    // THOSE two threads - unlike EmuSen.Mistress's console window, F4
    // and RunFrame() never run concurrently, because they share the one
    // emulation thread by construction.
    //
    // A third thread joins this picture as of the "Diana always live"
    // work: _consoleReaderThread (ConsoleReaderLoop below) is the sole
    // owner of ConsoleLineReader.ReadLine for the whole life of the
    // window, whether or not F4/a breakpoint is currently halted. While
    // running normally, it hands each typed line to _scheduler
    // (EmuSen.DianaOS.DianaOS.Bin.DianaOSInterpreterScheduler - see that class's own
    // comment for the full mechanism, since "diana isn't drowning" work
    // lifted it out of this file so EmuSen.Mistress could share it): a
    // read-only line runs right there on the reader thread (never
    // touching the emulation thread at all - the same "a read-only view
    // of live core state, unsynchronized, is an accepted race" precedent
    // CoretopWindow's own polling already relies on, though as of the
    // real-time-provider work most of what a read-only line actually
    // reads is a lock-free published snapshot, not truly unsynchronized
    // live state); anything else is queued and drained once per frame by
    // ProcessPendingConsoleCommands, alongside ProcessHotkeys. While
    // halted (RunDebugPrompt, F4/breakpoint), the reader thread instead
    // just forwards each line into the scheduler's halted-line queue and
    // RunDebugPrompt blocks reading from THAT - still fully synchronous
    // with RunFrame(), exactly as before this work, just fed by a
    // different thread than the one running the prompt loop. The
    // scheduler's own lock serializes the two things that can ever call
    // _debugCmd.Submit (the reader thread's fast path, and whichever
    // thread - emulation, via the frame-tick drain or RunDebugPrompt - is
    // executing a line for real) so a single DianaOSInterpreter instance,
    // which has real mutable state beyond core reads (variables, history,
    // $?), is never entered from two threads at once. It's held only for
    // the duration of one command's Execute, never across a blocking
    // read, so it can't turn into "the emulation thread waits on the
    // console" through the back door.
    public partial class GameWindow : Window
    {
        private readonly VenusCore _core;

        // Not readonly, unlike every other field this window was
        // originally handed - all three get rebuilt from scratch by
        // RebuildDebugTargetAndCommands() after a `core <name> <path>`
        // swap (HostAction.LoadCore, see SwapCore below), the same
        // "fresh SnesDebugTarget per load" discipline
        // EmuSen.Mistress/Views/MainWindow.axaml.cs's own LoadRom
        // already established - VenusCore.LoadRom rebuilds Cpu/Bus/
        // Renderer as brand-new objects in place, so anything still
        // holding the OLD ones (this class's own _debugTarget, and
        // DianaOSInterpreter, which captures its target at construction
        // with no UpdateTarget of its own - unlike EmuSen.Mistress's
        // console WINDOW) would otherwise go stale.
        private SnesDebugTarget _debugTarget = null!;

        // See `man tmux`.
        private readonly DianaOSSessionManager _sessions = new();

        // One scheduler per session - see `man tmux`.
        private readonly ConcurrentDictionary<DianaOSSession, DianaOSInterpreterScheduler> _schedulers = new();
        private DianaOSInterpreter _debugCmd => _sessions.Current!.Interpreter;
        private DianaOSInterpreterScheduler _scheduler => _schedulers[_sessions.Current!];

        private FrameRecorder _frameRecorder = null!;

        // Reused, unchanged, across every RebuildDebugTargetAndCommands()
        // call - none of these commands' own delegates depend on
        // anything that changes across a swap (CoretopCommand/
        // StateCommand close over _core/DebugWindows, both stable across
        // reloads; CoreCommand's own registry is static data), only on
        // whatever _debugTarget DianaOSInterpreter.CreateDefault is
        // handed each time.
        private readonly IEnumerable<IDianaOSCommand> _extraCommands;

        private readonly string _statePath;

        private readonly GamepadBindingMap _gamepadBindings = GamepadBindingMap.Load();
        private readonly GamepadManager _gamepad;
        private readonly AudioPlayer _audioPlayer = new();
        private readonly DispatcherTimer _gamepadTimer;

        // Keyboard and gamepad are tracked separately and combined with OR
        // logic - matches EmuSen.Mistress's own MainWindow convention
        // ("either device works at any time, no need to pick one").
        private readonly bool[] _keyboardHeld = new bool[Enum.GetValues<SnesButton>().Length];
        private readonly bool[] _gamepadHeld = new bool[Enum.GetValues<SnesButton>().Length];
        private bool _mirrorPlayer1ToPlayer2;

        private readonly DebugTools.BoundedTrace _bgScrollTrace = new();

        private volatile bool _running = true;
        private readonly Thread _emuThread;

        // See this file's own header comment on the "Diana always live"
        // threading model. _consoleReaderThread is started once, in the
        // constructor, and runs for the window's whole life - it isn't
        // tied to F4 the way console reading used to be.
        private readonly Thread _consoleReaderThread;

        // Every non-gameplay hotkey (F1-F9, O, P) - edge-detected in
        // OnKeyDown below (never on an OS key-repeat) and consumed once
        // per RunFrame() iteration on the emulation thread, the same
        // cadence the old Raylib polling loop's RunHotkeys ran at. Plain
        // volatile bool, not Interlocked: exactly one writer (this
        // window's UI-thread KeyDown handler) and one reader/clearer
        // (EmulationLoop) per flag - the same single-writer/single-reader
        // race EmuSen.Mistress's own ApplyButtonState already accepts for
        // continuous button state.
        private volatile bool _requestSummary;
        private volatile bool _requestDumpOam;
        private volatile bool _requestScreenshot;
        private volatile bool _requestDebugPrompt;
        private volatile bool _requestSaveState;
        private volatile bool _requestLoadState;
        private volatile bool _requestToggleRecording;
        private volatile bool _requestMirrorToggle;
        private volatile bool _requestDumpBackdrop;
        private volatile bool _requestStartBgScrollTrace;

        // Held, not edge-triggered - see EmuSen_Rewind_And_FastForward.md §4.
        private volatile bool _turboHeld;
        private volatile bool _rewindHeld;

        private readonly EmuSen.Common.SpeedController _speed = new();
        private readonly EmuSen.Common.RewindBuffer _rewind = new() { Enabled = true };

        private readonly HashSet<Key> _heldPhysicalKeys = new();

        // 'feed's own way back into the F4 prompt without needing this
        // window focused - see ArmFeedWatch's own comment below (ported
        // from the old Program.cs unchanged).
        private bool _feedWatchActive;

        // Coalescing hand-off from the emulation thread to the UI thread -
        // same pattern as EmuSen.Serenity.FramePresenter/
        // EmuSen.Mistress's own MainWindow.SubmitFrame, just driven
        // straight against GameFrame (this window's own GameFrameControl)
        // instead of a separate presenter object - see this file's own
        // header comment for why.
        private sealed class FrameData
        {
            public required byte[] Rgba;
            public required int Width;
            public required int Height;
        }
        private FrameData? _pendingFrame;
        private int _presentScheduled;

        public GameWindow(VenusCore core, IEnumerable<IDianaOSCommand> extraCommands, string statePath)
        {
            InitializeComponent();

            _core = core;
            _extraCommands = extraCommands;
            _statePath = statePath;

            RebuildDebugTargetAndCommands();
            // Power-on watch registration (Yoshi/coin investigation) -
            // only on this very first load, not re-added by SwapCore on a
            // later `core` swap - matches the same "shell-level state
            // resets on reload" precedent RebuildDebugTargetAndCommands's
            // own fresh SnesDebugTarget already establishes for watches/
            // breakpoints/cheats in general.
            _debugTarget.Watches.AddWatch("WRAM", 0x8000, 0x1800);
            _debugTarget.Watches.AddWatch("WRAM", 0x0D80, 0x0080);

            Title = GraphicsSettings.WindowTitle;
            Width = GraphicsSettings.WindowWidth;
            Height = GraphicsSettings.WindowHeight;
            CanResize = GraphicsSettings.WindowResizable;

            _gamepad = new GamepadManager(_gamepadBindings);
            _gamepadTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1.0 / 60.0) };
            _gamepadTimer.Tick += (_, _) => PollGamepad();
            _gamepadTimer.Start();

            KeyDown += OnKeyDown;
            KeyUp += OnKeyUp;
            Closing += (_, _) => Shutdown();

            _emuThread = new Thread(EmulationLoop) { IsBackground = true, Name = "EmuSen-Emulation" };
            _emuThread.Start();

            _consoleReaderThread = new Thread(ConsoleReaderLoop) { IsBackground = true, Name = "EmuSen-ConsoleReader" };
            _consoleReaderThread.Start();
        }

        // Sole owner of ConsoleLineReader.ReadLine for the whole life of
        // this window - see this file's own header comment. Runs on its
        // own dedicated thread so a blocking console read is never on the
        // emulation thread's critical path while running normally.
        private void ConsoleReaderLoop()
        {
            while (_running)
            {
                // Captured once per line, never re-read mid-line - see
                // this field's own comment on why (a `core` swap
                // reassigning it partway through a line's own
                // classify-then-execute sequence must not split that
                // sequence across two different interpreter instances).
                DianaOSInterpreter shell = _debugCmd;

                if (!_scheduler.Halted) Console.Write("DianaOS $ ");

                string[] historySnapshot = _scheduler.SnapshotHistory(shell);

                string? line;
                try { line = ConsoleLineReader.ReadLine(historySnapshot); }
                catch (Exception ex)
                {
                    // No real console to read from (e.g. stdin closed
                    // out from under this thread during shutdown) - stop
                    // quietly rather than spinning on a repeating error.
                    Console.WriteLine($"[CONSOLE] Reader thread stopped: {ex.Message}");
                    return;
                }
                // EOF (stdin closed/redirected input exhausted) - nothing
                // more will ever come from Console.ReadLine again, so
                // stop rather than busy-looping on an instantly-null
                // read. If this happens while halted, RunDebugPrompt's
                // own _scheduler.TakeHaltedLine() below simply has nothing
                // left to receive - bounded by Shutdown's own timeout on
                // window close, same as any other "prompt never got an
                // answer" case already was before this thread existed.
                if (line is null) return;

                var result = _scheduler.SubmitFromAnyThread(shell, line);
                if (result is { } r) Console.WriteLine(r.Output);
            }
        }

        // Drains lines the reader thread queued because they weren't
        // fast-path-eligible (mutating, or not a single bare simple
        // command) - called once per frame from EmulationLoop, alongside
        // ProcessHotkeys, so a mutating command typed at the live
        // terminal runs inline on the emulation thread's own next tick
        // instead of needing F4. Returns true on HostAction.Shutdown,
        // matching ProcessHotkeys' own return-true-means-close contract.
        private bool ProcessPendingConsoleCommands()
        {
            foreach (var (_, output, action) in _scheduler.DrainPending(_debugCmd))
            {
                Console.WriteLine(output);

                if (action is HostAction.Shutdown) return true;
                if (action is HostAction.LoadCore loadCore) { SwapCore(loadCore.RomPath); continue; }
                if (action is HostAction.SwitchSession) SyncSchedulersToSessions();
                // HostAction.Resume/Step with nothing halted: nothing to
                // resume from (Resume is a no-op) / StepCommand already
                // armed the single-step itself, which the next RunFrame()
                // iteration's own IsHaltedAtBreakpoint check picks up the
                // same way an F4-armed step already does.
            }
            return false;
        }

        private void OnKeyDown(object? sender, KeyEventArgs e)
        {
            if (!_heldPhysicalKeys.Add(e.Key)) return; // OS key-repeat, not a fresh press

            if (HotaruKeyMap.TryGetButton(e.Key, out SnesButton button))
            {
                _keyboardHeld[(int)button] = true;
                ApplyButtonState(button);
                return;
            }

            switch (e.Key)
            {
                case Key.F1: _requestSummary = true; break;
                case Key.F2: _requestDumpOam = true; break;
                case Key.F3: _requestScreenshot = true; break;
                case Key.F4: _requestDebugPrompt = true; break;
                case Key.F5: _requestSaveState = true; break;
                case Key.F6: _requestToggleRecording = true; break;
                case Key.F7: _requestMirrorToggle = true; break;
                case Key.F8: CycleShaderEffect(); break; // pure UI-side (GameFrame.ActiveEffect) - no core state touched, safe to act on immediately
                case Key.F9: _requestLoadState = true; break;
                case Key.O: _requestDumpBackdrop = true; break;
                case Key.P: _requestStartBgScrollTrace = true; break;
                case Key.Tab: _turboHeld = true; e.Handled = true; break; // Handled or Avalonia steals Tab for focus traversal
                case Key.Back: _rewindHeld = true; break;
            }
        }

        private void OnKeyUp(object? sender, KeyEventArgs e)
        {
            _heldPhysicalKeys.Remove(e.Key);
            if (HotaruKeyMap.TryGetButton(e.Key, out SnesButton button))
            {
                _keyboardHeld[(int)button] = false;
                ApplyButtonState(button);
                return;
            }

            switch (e.Key)
            {
                case Key.Tab: _turboHeld = false; break;
                case Key.Back: _rewindHeld = false; break;
            }
        }

        private void ApplyButtonState(SnesButton button)
        {
            bool held = _keyboardHeld[(int)button] || _gamepadHeld[(int)button];
            _core.Bus!.Input.SetButton(button, held);
            if (_mirrorPlayer1ToPlayer2) _core.Bus.Input.SetButton(button, held, controller: 2);
        }

        private void PollGamepad()
        {
            _gamepad.Poll();
            foreach (SnesButton button in Enum.GetValues<SnesButton>())
            {
                bool held = _gamepad.IsPressed(button);
                if (held != _gamepadHeld[(int)button])
                {
                    _gamepadHeld[(int)button] = held;
                    ApplyButtonState(button);
                }
            }
        }

        // Cycles None -> Scanlines -> Crt -> None - reuses
        // FramePresenter's own static NextEffect, not FramePresenter
        // itself (see this file's header comment).
        private void CycleShaderEffect()
        {
            ShaderEffect effect = FramePresenter.NextEffect(GameFrame.ActiveEffect);
            GameFrame.ActiveEffect = effect;
            Console.WriteLine($"[SHADER] Active effect: {effect}");
        }

        // Rebuilds _debugTarget/_frameRecorder and every session - see
        // `man core`/`man tmux`.
        private void RebuildDebugTargetAndCommands()
        {
            _debugTarget = new SnesDebugTarget(_core.Cpu!, _core.Bus!, _core.Renderer!,
                () => (_core.LastFrameCpuSpc700Ms, _core.LastFramePpuMs, _core.LastFrameHdmaMs));
            _frameRecorder = new FrameRecorder(_debugTarget);

            if (_sessions.Sessions.Count == 0)
            {
                var initial = new DianaOSSession("main", BuildInterpreter());
                _sessions.RegisterInitial(initial);
                _schedulers[initial] = new DianaOSInterpreterScheduler();
                return;
            }

            _sessions.RebuildAll(BuildInterpreter);
            SyncSchedulersToSessions();
        }

        private DianaOSInterpreter BuildInterpreter() =>
            DianaOSInterpreter.CreateDefault(_debugTarget, _extraCommands,
                new EmuSen.Cores.Nintendo.Venus.Cheats.ActionReplayCheatCodec(),
                new EmuSen.Cores.Nintendo.Venus.Cheats.GameGenieCheatCodec(),
                new EmuSen.Cores.Nintendo.Venus.Debug.VenusCpuTraceSwitch(),
                _sessions);

        // Syncs _schedulers to whatever sessions currently exist - see `man tmux`.
        private void SyncSchedulersToSessions()
        {
            var live = new HashSet<DianaOSSession>(_sessions.Sessions);
            foreach (DianaOSSession stale in _schedulers.Keys.Where(s => !live.Contains(s)).ToList())
            {
                _schedulers.TryRemove(stale, out _);
            }
            foreach (DianaOSSession session in _sessions.Sessions)
            {
                if (!_schedulers.ContainsKey(session)) _schedulers[session] = new DianaOSInterpreterScheduler();
            }
        }

        // Reached from RunDebugPrompt's own dispatch loop when
        // EmuSen.DianaOS.DianaOS.Bin.Commands.CoreCommand signals HostAction.LoadCore
        // - the actual ROM swap this migration's whole point was to
        // enable (Hotaru had NO way to change ROMs mid-session before
        // this). coreName isn't needed here at all: Hotaru only has one
        // ICore implementation today (VenusCore), so every registered
        // core alias ('venus'/'snes') reloads the exact same way -
        // CoreCommand already validated the name/file/extension before
        // ever emitting this action. Deliberately this narrow (reload the
        // one existing core in place, no real multi-core dispatch) rather
        // than building toward a general ICoreSession abstraction ahead
        // of a second core actually existing - see
        // EmuSen_Launcher_Multicore_Gameplan.md's own guidance on that.
        private void SwapCore(string romPath)
        {
            if (_frameRecorder.IsRecording)
            {
                Console.WriteLine("[CORE] Refusing to swap ROM while a recording is in progress - stop it first (F6).");
                return;
            }

            _core.LoadRom(romPath);
            _rewind.Clear(); // a discontinuous jump - see §1.4
            _audioPlayer.RateControl.Reset();
            RebuildDebugTargetAndCommands();
            // Without this, an already-open `coretop -w` window would
            // silently keep showing the OLD, now-discarded target forever
            // - it has no timer of its own that would ever notice a swap
            // happened, only a refresh timer that re-polls whatever
            // target it was last told about. No-ops if coretop was never
            // opened this session - see that method's own comment.
            DebugWindows.UpdateCoretopWindowTargetIfOpen(_debugTarget);
            Console.WriteLine($"[CORE] Loaded: {romPath}");
        }

        // Runs entirely off the UI thread - see this file's own header
        // comment on the threading model.
        private void EmulationLoop()
        {
            Stopwatch clock = Stopwatch.StartNew();
            TimeSpan nextTick = clock.Elapsed;

            try
            {
                while (_running)
                {
                    _speed.SetTurbo(_turboHeld);
                    TimeSpan interval = _speed.FrameInterval(_core.FrameRateHz);
                    nextTick += interval;

                    // Takes over the frame entirely - see EmuSen_Rewind_And_FastForward.md §4.
                    if (_rewindHeld)
                    {
                        _core.SkipRendering = false;
                        if (!_rewind.Rewind(_core)) nextTick = clock.Elapsed;
                        // No RunFrame() ran, so these would otherwise be stale - see §3.
                        _debugTarget.RefreshProviders();
                        _core.DequeueAudioSamples(int.MaxValue);
                        _audioPlayer.RateControl.Reset(); // skipped content - see EmuSen_Audio_Sync.md §3.2
                        SubmitFrame(_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight);
                        if (ProcessPendingConsoleCommands()) { RequestClose(); return; }
                        SleepUntil(nextTick, clock);
                        continue;
                    }

                    _core.SkipRendering = !_speed.ShouldRender(_core.TotalFrames);

                    _core.RunFrame();

                    // Publishes this frame's register/sprite/palette/audio/
                    // hardware-load snapshots for any thread to read
                    // (Diana's console-reader fast path in particular) via
                    // IDebugTarget's provider properties - see
                    // EmuSen.Cauldron.IRealtimeProvider's own comment. Must
                    // run here, on the emulation thread that just produced
                    // this frame's state, not from the console-reader
                    // thread or anywhere else.
                    _debugTarget.RefreshProviders();

                    // A breakpoint (or an armed single-step) halted
                    // RunFrame() before it finished this frame - skip
                    // presenting/hotkeys this iteration and go straight to
                    // the same prompt F4 already uses, same as the old
                    // Raylib loop's own breakpoint handling.
                    if (_core.IsHaltedAtBreakpoint)
                    {
                        Console.WriteLine($"\n[BREAKPOINT] Halted at ${_core.HaltedAddress:X6}");
                        Console.WriteLine(_debugTarget.GetSummaryText());
                        if (RunDebugPrompt()) { RequestClose(); return; }
                        nextTick = clock.Elapsed; // don't burst-catch-up for time spent at the prompt
                        continue;
                    }

                    _rewind.OnFrameCompleted(_core);

                    // Drained, not just unpumped, when speed outruns the device - see §2.3.
                    if (_speed.ShouldPlayAudio)
                    {
                        _audioPlayer.Pump(_core);
                    }
                    else
                    {
                        _core.DequeueAudioSamples(int.MaxValue);
                        _audioPlayer.RateControl.Reset(); // skipped content - see EmuSen_Audio_Sync.md §3.2
                    }

                    // Nothing new was drawn on a skipped frame.
                    if (!_core.SkipRendering) SubmitFrame(_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight);

                    if (ProcessHotkeys()) { RequestClose(); return; }
                    if (ProcessPendingConsoleCommands()) { RequestClose(); return; }

                    TimeSpan remaining = nextTick - clock.Elapsed;
                    if (remaining > TimeSpan.Zero)
                    {
                        SleepUntil(nextTick, clock);
                    }
                    else
                    {
                        // Fell behind - resync to "now" instead of trying to
                        // burst-catch-up, which would just run a pile of
                        // frames back-to-back with no pacing at all.
                        nextTick = clock.Elapsed;
                    }
                }
            }
            catch (Exception ex)
            {
                // Matches the old console build's own behavior: halt and
                // print rather than trying to recover. SaveSram() isn't
                // called here either, same as before this migration - the
                // core's own periodic autosave (inside RunFrame() itself)
                // is what covers a crash, not this catch block.
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
                RequestClose();
            }
        }

        // Spin-waits rather than Thread.Sleep - see EmuSen.Mistress's own
        // identical SleepUntil for the measured reasoning (Sleep's wakeup
        // latency was bigger than the 60fps budget allows for).
        private static void SleepUntil(TimeSpan target, Stopwatch clock)
        {
            while (clock.Elapsed < target)
            {
                Thread.SpinWait(100);
            }
        }

        private void RequestClose()
        {
            _running = false;
            Dispatcher.UIThread.Post(Close);
        }

        // Runs once per RunFrame() iteration on the emulation thread -
        // same cadence the old Raylib loop's RunHotkeys had, just
        // consuming edge-triggered request flags set by OnKeyDown (UI
        // thread) instead of polling Raylib.IsKeyPressed itself. Returns
        // true if a shutdown was requested (F4's 'shutdown', or 'feed's
        // own Ctrl+C reopening the prompt and getting 'shutdown' there).
        private bool ProcessHotkeys()
        {
            // 'feed's own way back into the prompt - see ArmFeedWatch's
            // own comment. Guarded on IsInputRedirected the same way
            // coretop's own dashboard is - KeyAvailable throws if there's
            // no real console to poll.
            if (_feedWatchActive && !Console.IsInputRedirected && Console.KeyAvailable)
            {
                ConsoleKeyInfo key = Console.ReadKey(intercept: true);
                if ((key.Modifiers & ConsoleModifiers.Control) != 0 && key.Key == ConsoleKey.C)
                {
                    Console.WriteLine();
                    if (RunDebugPrompt()) return true;
                }
            }

            // Every frame, cheap no-op when not recording.
            _frameRecorder.CaptureFrame(path => FrameImageWriter.SavePng(_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight, path));

            if (_requestToggleRecording) { _requestToggleRecording = false; ToggleRecording(); }
            if (_requestStartBgScrollTrace) { _requestStartBgScrollTrace = false; StartBgScrollTrace(); }
            if (_requestDumpBackdrop) { _requestDumpBackdrop = false; _core.Renderer!.DumpBackdropAndWindowDebugInfo(_core.Bus!.Ppu, _core.TotalFrames); }
            if (_requestMirrorToggle) { _requestMirrorToggle = false; ToggleMirror(); }
            if (_requestSummary) { _requestSummary = false; Console.WriteLine(_debugTarget.GetSummaryText()); }
            if (_requestDumpOam) { _requestDumpOam = false; _core.Renderer!.DumpActiveOam(_core.Bus!.Ppu); }
            if (_requestDebugPrompt) { _requestDebugPrompt = false; if (RunDebugPrompt()) return true; }
            if (_requestScreenshot) { _requestScreenshot = false; TakeScreenshot(); }
            if (_requestSaveState) { _requestSaveState = false; SaveState(_statePath); }
            if (_requestLoadState) { _requestLoadState = false; LoadState(_statePath); }

            if (_bgScrollTrace.ShouldLog())
            {
                Console.WriteLine($"[BG SCROLL] Frame {_core.TotalFrames}: BG1 X={_core.Bus!.Ppu.BgScrollX[0]} Y={_core.Bus.Ppu.BgScrollY[0]}  |  BG2 X={_core.Bus.Ppu.BgScrollX[1]} Y={_core.Bus.Ppu.BgScrollY[1]}");
                if (!_bgScrollTrace.IsActive) DebugSettings.AllScrollWriteLogging = false;
            }

            return false;
        }

        private void TakeScreenshot()
        {
            string coreLogDir = Path.Combine(DianaOSSandbox.LogsDirectory, _debugTarget.CoreName);
            Directory.CreateDirectory(coreLogDir);
            string baseName = $"screenshot_frame{_debugTarget.FrameCount}";
            string shotPath = Path.Combine(coreLogDir, baseName + ".png");
            string metaPath = Path.Combine(coreLogDir, baseName + ".txt");

            FrameImageWriter.SavePng(_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight, shotPath);
            File.WriteAllText(metaPath,
                $"Core: {_debugTarget.CoreName}\n" +
                $"Frame: {_debugTarget.FrameCount}\n" +
                $"Wall-clock: {DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}\n");

            Console.WriteLine($"[SCREENSHOT] Saved {shotPath} (Core={_debugTarget.CoreName} Frame={_debugTarget.FrameCount})");
        }

        private void SaveState(string path)
        {
            try
            {
                _core.SaveState(path);
                Console.WriteLine($"[STATE] Saved: {path}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[STATE] Save failed: {ex.Message}");
            }
        }

        private void LoadState(string path)
        {
            try
            {
                _core.LoadState(path);
                _rewind.Clear(); // a discontinuous jump - see §1.4
                _audioPlayer.RateControl.Reset();
                Console.WriteLine($"[STATE] Loaded: {path}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[STATE] Load failed: {ex.Message}");
            }
        }

        private void ToggleRecording()
        {
            if (_frameRecorder.IsRecording)
            {
                Console.WriteLine($"[RECORD] Stopped: {_frameRecorder.SessionDir}");
                _frameRecorder.Stop();
            }
            else
            {
                string dir = _frameRecorder.Start(Path.Combine(DianaOSSandbox.LogsDirectory, _debugTarget.CoreName, "Recordings"));
                Console.WriteLine($"[RECORD] Started -> {dir}");
            }
        }

        private void ToggleMirror()
        {
            _mirrorPlayer1ToPlayer2 = !_mirrorPlayer1ToPlayer2;
            Console.WriteLine($"[INPUT] Mirror Player 1 -> Player 2: {(_mirrorPlayer1ToPlayer2 ? "ON" : "OFF")}");
        }

        private void StartBgScrollTrace()
        {
            _bgScrollTrace.Start(300);
            DebugSettings.AllScrollWriteLogging = true;
            Console.WriteLine("[BG SCROLL] --- Starting 300-frame scroll trace ---");
        }

        // Shared by the F4 hotkey and the emulation loop's own
        // breakpoint-halt check - see the old Program.cs's own comment on
        // why (both funnel through the exact same interactive command
        // loop rather than keeping two copies in sync). Blocks THIS
        // (emulation) thread on _scheduler.TakeHaltedLine() - never the UI
        // thread - which is exactly what lets the window stay movable/
        // resizable/closable while F4 is open, unlike the old Raylib
        // build where the window and this prompt shared one thread.
        // Doesn't read the console directly any more - see this file's
        // own header comment on why _consoleReaderThread now owns that,
        // and forwards lines here (instead of handling them itself)
        // whenever _scheduler.Halted is set.
        private bool RunDebugPrompt()
        {
            DisarmFeedWatch();
            _scheduler.Halted = true;
            try
            {
                Console.WriteLine("--- DianaOS (type 'help', 'resume' to resume, 'feed'/'feed -w' to resume and watch gameplay, 'shutdown' to quit, 'step'/'s' to single-step, 'core <name> <path>' to swap ROMs) ---");
                while (true)
                {
                    Console.Write(_debugCmd.IsAwaitingMoreInput ? "> " : "DianaOS #: ");
                    string line = _scheduler.TakeHaltedLine();
                    string trimmed = line.Trim();

                    if (!_debugCmd.IsAwaitingMoreInput)
                    {
                        // Empty-line-means-resume stays pre-dispatch, same as
                        // before - an empty line isn't really "a command," the
                        // same way SubmitCore itself already treats one as a
                        // no-op, so there's nothing gained by routing it
                        // through ResumeCommand too.
                        if (trimmed.Length == 0)
                        {
                            break;
                        }
                        if (trimmed.Equals("feed", StringComparison.OrdinalIgnoreCase) || trimmed.StartsWith("feed ", StringComparison.OrdinalIgnoreCase))
                        {
                            string[] feedParts = trimmed.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                            bool windowed = feedParts.Length >= 2 && feedParts[1].Equals("-w", StringComparison.OrdinalIgnoreCase);
                            if (windowed)
                            {
                                DebugWindows.ShowFeedWindow(() => (_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight));
                                Console.WriteLine("[FEED] Opened in a separate window.");
                            }
                            ArmFeedWatch();
                            Console.WriteLine("Resuming - press Ctrl+C in this terminal to reopen the prompt.");
                            break;
                        }
                    }
                    // 'resume'/'continue'/'c', 'shutdown'/'quit', 'step'/'s',
                    // and 'core <name> <path>' are all real DianaOS commands
                    // now (EmuSen.DianaOS.DianaOS.Bin.Commands.ResumeCommand/
                    // ShutdownCommand/StepCommand/CoreCommand) - Submit's own
                    // HostAction is what lets them reach back out to this
                    // loop's control flow, the same thing the old hand-rolled
                    // string matches used to do directly. A successful swap
                    // resumes gameplay immediately against the new ROM, same
                    // as Resume/Step - SwapCore itself prints why if it
                    // refuses (a recording in progress). Locked (see this
                    // file's own header comment) so this can never overlap a
                    // fast-path Submit from the reader thread - only
                    // relevant for the brief window right as a halt begins.
                    // Captured before the call, not read again after -
                    // _scheduler is computed off _sessions.Current, which
                    // a `tmux switch`/`tmux new` inside this very Submit
                    // call changes, so the receiver has to be pinned to
                    // whichever session was actually halted here.
                    DianaOSInterpreterScheduler haltedScheduler = _scheduler;
                    (_, string output, HostAction? action) = haltedScheduler.SubmitLocked(_debugCmd, trimmed);
                    Console.WriteLine(output);
                    if (action is HostAction.Shutdown) return true;
                    if (action is HostAction.SwitchSession)
                    {
                        // Halted is per-scheduler, not per-window - without
                        // this, haltedScheduler (the OLD session) stays
                        // stuck thinking it's still halted forever, since
                        // this method's own finally block only ever clears
                        // whatever _scheduler resolves to when IT runs, and
                        // that's the NEW session by then.
                        haltedScheduler.Halted = false;
                        SyncSchedulersToSessions();
                        _scheduler.Halted = true;
                        continue;
                    }
                    if (action is HostAction.Resume or HostAction.Step) break;
                    if (action is HostAction.LoadCore loadCore) { SwapCore(loadCore.RomPath); break; }
                }
                Console.WriteLine("--- Resuming ---");
                return false;
            }
            finally
            {
                _scheduler.Halted = false;
            }
        }

        private void ArmFeedWatch()
        {
            try { Console.TreatControlCAsInput = true; } catch { }
            _feedWatchActive = true;
        }

        private void DisarmFeedWatch()
        {
            if (!_feedWatchActive) return;
            try { Console.TreatControlCAsInput = false; } catch { }
            _feedWatchActive = false;
        }

        private void SubmitFrame(byte[] rgba, int width, int height)
        {
            Interlocked.Exchange(ref _pendingFrame, new FrameData { Rgba = rgba, Width = width, Height = height });
            if (Interlocked.CompareExchange(ref _presentScheduled, 1, 0) == 0)
            {
                Dispatcher.UIThread.Post(PresentPendingFrame);
            }
        }

        private void PresentPendingFrame()
        {
            try
            {
                FrameData? frame = Interlocked.Exchange(ref _pendingFrame, null);
                if (frame is null) return;
                GameFrame.UpdateFrame(frame.Rgba, frame.Width, frame.Height);
            }
            finally
            {
                Interlocked.Exchange(ref _presentScheduled, 0);
            }
        }

        private void Shutdown()
        {
            _gamepadTimer.Stop();
            _running = false;

            // Bounded, not indefinite: the emulation thread can be
            // blocked inside RunDebugPrompt()'s _scheduler.TakeHaltedLine() if
            // F4's prompt is open when the window is closed - unlike the
            // old Raylib build, where the window and the console prompt
            // shared one thread and this situation could never even be
            // reached (see this file's own header comment on why F4
            // lives on the emulation thread at all). Waiting forever here
            // would freeze the UI thread until someone types something at
            // the terminal; 500ms covers the overwhelmingly common case
            // (the loop notices _running=false within one frame interval)
            // without risking an indefinite hang for the rare
            // "closed mid-prompt" case.
            _emuThread.Join(TimeSpan.FromMilliseconds(500));

            // _consoleReaderThread is very likely blocked in
            // ConsoleLineReader.ReadLine right now, with no cooperative
            // way to cancel a blocking console read - same
            // accept-it's-background-and-move-on treatment as _emuThread
            // above, just with no Join at all (IsBackground = true
            // already means process exit won't wait on it either way, and
            // there's nothing further this thread needs to have finished
            // before the window can close). _running is already false by
            // this point (set above), so it'll notice and stop on its
            // very next loop iteration if it isn't blocked on a read.

            _core.SaveSram();
            _gamepad.Dispose();
            _audioPlayer.Dispose();
        }
    }
}
