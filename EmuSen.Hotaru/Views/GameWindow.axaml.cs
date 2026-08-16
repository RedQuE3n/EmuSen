using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using Avalonia.Controls;
using EmuSen.LunaP.Windowing;
using Avalonia.Input;
using Avalonia.Threading;
using EmuSen.LunaP.Threading;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Graphics;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Hotaru.Imaging;
using EmuSen.Hotaru.Input;
using EmuSen.Serenity;
using EmuSen.Galaxia.Input;
using EmuSen.Audio;

namespace EmuSen.Hotaru.Views
{
    // Hotaru's game window and its three threads - see EmuSen_Frontend_Driver.md §1, §2 and §3f.
    public partial class GameWindow : Window, ILiveShell
    {
        private readonly ICore _core;

        // The pointer gets out of the way over the game - see EmuSen_Frontend_Driver.md §3d.
        private IdleCursor? _idleCursor;
        private FileDrop? _fileDrop;

        // Set by a drop on the UI thread, taken by ProcessHotkeys - SwapCore runs there. See §3d.
        private volatile string? _droppedRom;

        // The Venus-only hotkeys below need the real core; null for any other console.
        private VenusCore? Venus => _core as VenusCore;

        // Rebuilt per load, which is why it is not readonly - see EmuSen_Frontend_Driver.md §3f.
        private IDebugTarget _debugTarget = null!;

        // The rest of the loaded core's wiring - codecs, trace switch - see EmuSen_Multicore.md §4.
        private CoreBundle? _bundle;

        // See `man tmux`.
        private readonly DianaOSSessionManager _sessions = new();

        // One scheduler per session - see `man tmux`.
        private readonly ConcurrentDictionary<DianaOSSession, DianaOSInterpreterScheduler> _schedulers = new();
        private DianaOSInterpreter _debugCmd => _sessions.Current!.Interpreter;
        private DianaOSInterpreterScheduler _scheduler => _schedulers[_sessions.Current!];

        private FrameRecorder _frameRecorder = null!;

        // Reused unchanged across every rebuild - see EmuSen_Frontend_Driver.md §3f.
        private readonly IEnumerable<IDianaOSCommand> _extraCommands;

        private readonly string _statePath;

        private readonly GamepadBindings _gamepadBindings =
            GamepadBindings.Load(EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console));
        private readonly GamepadManager _gamepad;
        // Endymion is a leaf and reads no globals, so the settings come from here - see EmuSen_Audio_Sync.md §7.1.
        private readonly AudioPlayer _audioPlayer = new(
            AudioSettings.SampleRate, AudioSettings.OutputTargetLatencyMs, AudioSettings.RateControlMaxDeviation);
        private readonly DispatcherTimer _gamepadTimer;

        // Tracked separately and OR'd: either device works at any time.
        private readonly bool[] _keyboardHeld = new bool[Enum.GetValues<PadButton>().Length];
        private readonly bool[] _gamepadHeld = new bool[Enum.GetValues<PadButton>().Length];
        private bool _mirrorPlayer1ToPlayer2;

        private readonly DebugTools.BoundedTrace _bgScrollTrace = new();

        private volatile bool _running = true;
        private readonly Thread _emuThread;

        // Runs for the window's whole life, not just while halted - see EmuSen_Frontend_Driver.md §1.
        private readonly Thread _consoleReaderThread;

        // One writer (OnKeyDown), one reader/clearer (EmulationLoop) - see EmuSen_Frontend_Driver.md §2.
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

        // 'feed's way back into the prompt without window focus - see `man feed`.
        private bool _feedWatchActive;

        // Coalescing hand-off to the UI thread - see EmuSen_Serenity.md §4.
        private sealed class FrameData
        {
            public required byte[] Rgba;
            public required int Width;
            public required int Height;
        }
        // Was written out here, in Mistress and in Serenity, all three with one defect - LunaP.md §22.1.
        private readonly Latest<FrameData> _frames;

        public GameWindow(ICore core, IEnumerable<IDianaOSCommand> extraCommands, string statePath)
        {
            _frames = new Latest<FrameData>(PresentPendingFrame);
            InitializeComponent();

            _core = core;
            _extraCommands = extraCommands;
            _statePath = statePath;

            RebuildDebugTargetAndCommands();
            // First load only; a swap deliberately does not re-add them - see EmuSen_Frontend_Driver.md §3f.
            _debugTarget.Watches.AddWatch("WRAM", 0x8000, 0x1800);
            _debugTarget.Watches.AddWatch("WRAM", 0x0D80, 0x0080);

            Title = GraphicsSettings.WindowTitle;
            Width = GraphicsSettings.WindowWidth;
            Height = GraphicsSettings.WindowHeight;
            CanResize = GraphicsSettings.WindowResizable;

            _gamepad = new GamepadManager(_gamepadBindings.For(core.CoreName));
            _gamepadTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1.0 / 60.0) };
            _gamepadTimer.Tick += (_, _) => PollGamepad();
            _gamepadTimer.Start();

            KeyDown += OnKeyDown;
            KeyUp += OnKeyUp;

            // The whole window, because the whole window is the game here.
            _idleCursor = new IdleCursor(this);
            // Queued rather than loaded: SwapCore belongs to the emulation thread.
            _fileDrop = new FileDrop(this, paths => _droppedRom = paths[0]) { Accept = paths => paths.Count == 1 };

            Closing += (_, _) =>
            {
                // A cursor left hidden by an object nobody disposed is a pointer gone for good.
                _idleCursor?.Dispose();
                _fileDrop?.Dispose();
                Shutdown();
            };

            _emuThread = new Thread(EmulationLoop) { IsBackground = true, Name = "EmuSen-Emulation" };
            _emuThread.Start();

            _consoleReaderThread = new Thread(ConsoleReaderLoop) { IsBackground = true, Name = "EmuSen-ConsoleReader" };
            _consoleReaderThread.Start();
        }

        // Sole owner of ConsoleLineReader.ReadLine - see EmuSen_Frontend_Driver.md §3f.
        private void ConsoleReaderLoop()
        {
            while (_running)
            {
                // Once per line, never re-read mid-line - see EmuSen_Frontend_Driver.md §3f.
                DianaOSInterpreter shell = _debugCmd;

                if (!_scheduler.Halted) Console.Write("DianaOS $ ");

                string[] historySnapshot = _scheduler.SnapshotHistory(shell);

                string? line;
                try { line = ConsoleLineReader.ReadLine(historySnapshot); }
                catch (Exception ex)
                {
                    // Stop rather than spin on a repeating error - see EmuSen_Frontend_Driver.md §3f.
                    Console.WriteLine($"[CONSOLE] Reader thread stopped: {ex.Message}");
                    return;
                }
                // EOF: nothing more will ever arrive - see EmuSen_Frontend_Driver.md §3f.
                if (line is null) return;

                var result = _scheduler.SubmitFromAnyThread(shell, line);
                if (result is { } r) EmitShellOutput(r.Output);
            }
        }

        // ILiveShell: the same interpreter the terminal drives, lent to a shell window - see EmuSen_Frontend_Driver.md §3c.
        public event Action<string>? Output;

        bool ILiveShell.IsAwaitingMoreInput => _debugCmd.IsAwaitingMoreInput;

        string[] ILiveShell.SnapshotHistory() => _scheduler.SnapshotHistory(_debugCmd);

        void ILiveShell.Submit(string line)
        {
            var result = _scheduler.SubmitFromAnyThread(_debugCmd, line);
            if (result is { } r) EmitShellOutput(r.Output);
        }

        // Console when nobody is attached, so a terminal launch behaves exactly as it always did.
        private void EmitShellOutput(string output)
        {
            if (Output is { } sink) sink(output);
            else Console.WriteLine(output);
        }

        // Drains the reader thread's queue once per frame; true means close - see EmuSen_Frontend_Driver.md §1.
        private bool ProcessPendingConsoleCommands()
        {
            foreach (var (_, output, action) in _scheduler.DrainPending(_debugCmd))
            {
                EmitShellOutput(output);

                if (action is HostAction.Shutdown) return true;
                if (action is HostAction.LoadCore loadCore) { SwapCore(loadCore.RomPath); continue; }
                if (action is HostAction.SwitchSession) SyncSchedulersToSessions();
                // Resume with nothing halted is a no-op, and StepCommand already armed its own step.
            }
            return false;
        }

        private void OnKeyDown(object? sender, KeyEventArgs e)
        {
            if (!_heldPhysicalKeys.Add(e.Key)) return; // OS key-repeat, not a fresh press

            if (HotaruKeyMap.TryGetButton(e.Key, out PadButton button))
            {
                _keyboardHeld[(int)button] = true;
                ApplyButtonState(button);
                return;
            }

            // Through the table, so the help window and the dispatch cannot disagree - see §3e.
            if (!HotaruHotkeys.TryGetAction(e.Key, out HotaruHotkey action)) return;

            switch (action)
            {
                case HotaruHotkey.Summary: _requestSummary = true; break;
                case HotaruHotkey.DumpOam: _requestDumpOam = true; break;
                case HotaruHotkey.Screenshot: _requestScreenshot = true; break;
                case HotaruHotkey.DebugPrompt: _requestDebugPrompt = true; break;
                case HotaruHotkey.SaveState: _requestSaveState = true; break;
                case HotaruHotkey.ToggleRecording: _requestToggleRecording = true; break;
                case HotaruHotkey.MirrorToggle: _requestMirrorToggle = true; break;
                // Both are UI-side only, so they act here rather than being queued.
                case HotaruHotkey.ShowHotkeys: DebugWindows.ShowHotkeyHelpWindow(); break;
                case HotaruHotkey.CycleShader: CycleShaderEffect(); break; // pure UI-side (GameFrame.ActiveEffect) - no core state touched, safe to act on immediately
                case HotaruHotkey.LoadState: _requestLoadState = true; break;
                case HotaruHotkey.DumpBackdrop: _requestDumpBackdrop = true; break;
                case HotaruHotkey.StartBgScrollTrace: _requestStartBgScrollTrace = true; break;
                case HotaruHotkey.FastForward: _turboHeld = true; e.Handled = true; break; // Handled or Avalonia steals Tab for focus traversal
                case HotaruHotkey.Rewind: _rewindHeld = true; break;
            }
        }

        private void OnKeyUp(object? sender, KeyEventArgs e)
        {
            _heldPhysicalKeys.Remove(e.Key);
            if (HotaruKeyMap.TryGetButton(e.Key, out PadButton button))
            {
                _keyboardHeld[(int)button] = false;
                ApplyButtonState(button);
                return;
            }

            if (!HotaruHotkeys.TryGetAction(e.Key, out HotaruHotkey action)) return;

            switch (action)
            {
                case HotaruHotkey.FastForward: _turboHeld = false; break;
                case HotaruHotkey.Rewind: _rewindHeld = false; break;
            }
        }

        // Through ICore, not the bus - which console's pad this reaches is the core's business.
        private void ApplyButtonState(PadButton button)
        {
            bool held = _keyboardHeld[(int)button] || _gamepadHeld[(int)button];
            _core.SetButton(0, button, held);
            if (_mirrorPlayer1ToPlayer2) _core.SetButton(1, button, held);
        }

        private void PollGamepad()
        {
            _gamepad.Poll();
            foreach (PadButton button in Enum.GetValues<PadButton>())
            {
                bool held = _gamepad.IsPressed(button);
                if (held != _gamepadHeld[(int)button])
                {
                    _gamepadHeld[(int)button] = held;
                    ApplyButtonState(button);
                }
            }
        }

        // Reuses FramePresenter's static NextEffect, not FramePresenter itself - see EmuSen_Serenity.md §4.
        private void CycleShaderEffect()
        {
            ShaderEffect effect = FramePresenter.NextEffect(GameFrame.ActiveEffect);
            GameFrame.ActiveEffect = effect;
            Console.WriteLine($"[SHADER] Active effect: {effect}");
        }

        // Rebuilds _debugTarget/_frameRecorder and every session - see `man core` and `man tmux`.
        private void RebuildDebugTargetAndCommands()
        {
            _bundle = CoreFactory.Bundle(_core);
            _debugTarget = _bundle.DebugTarget;
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
                _bundle?.CheatAutoDetectCodec, _bundle?.CheatExplicitCodec, _bundle?.CpuTraceSwitch,
                _sessions,
                () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems);

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

        // Deliberately narrow: one core, reloaded in place - see EmuSen_Frontend_Driver.md §3f.
        private void SwapCore(string romPath)
        {
            if (_frameRecorder.IsRecording)
            {
                Console.WriteLine("[CORE] Refusing to swap ROM while a recording is in progress - stop it first (F6).");
                return;
            }

            _core.LoadRom(romPath);
            // The new console's pad, not the outgoing one's - see EmuSen_Input.md §5.1.
            _gamepad.Bindings = _gamepadBindings.For(_core.CoreName);
            _rewind.Clear(); // a discontinuous jump - see §1.4
            _audioPlayer.RateControl.Reset();
            RebuildDebugTargetAndCommands();
            // An open dashboard has a refresh timer, not a swap notification - see EmuSen_Frontend_Driver.md §3f.
            DebugWindows.UpdateCoretopWindowTargetIfOpen(_debugTarget);
            Console.WriteLine($"[CORE] Loaded: {romPath}");
        }

        // Runs entirely off the UI thread - see EmuSen_Frontend_Driver.md §1.
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

                    // Publishes this frame's snapshots, on the thread that produced them - see EmuSen_Cauldron.md.
                    _debugTarget.RefreshProviders();

                    // A breakpoint or armed step stopped RunFrame mid-frame; go straight to the prompt.
                    if (_core.IsHaltedAtBreakpoint)
                    {
                        Console.WriteLine($"\n[BREAKPOINT] Halted at ${_core.HaltedAddress:X6}");
                        Console.WriteLine(_debugTarget.GetSummaryText());
                        if (RunDebugPrompt()) { RequestClose(); return; }
                        nextTick = clock.Elapsed; // don't burst-catch-up for time spent at the prompt
                        continue;
                    }

                    _rewind.OnFrameCompleted(_core);

                    // Drained every frame either way, so a muted stretch cannot back the buffer up - see EmuSen_Audio_Sync.md §4.
                    short[] samples = _core.DequeueAudioSamples(int.MaxValue);
                    if (_speed.ShouldPlayAudio) _audioPlayer.Submit(samples, _core.AudioSampleRate);
                    else _audioPlayer.RateControl.Reset(); // skipped content - see EmuSen_Audio_Sync.md §3.2

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
                        // Resync to now rather than burst-catching up with no pacing at all.
                        nextTick = clock.Elapsed;
                    }
                }
            }
            catch (Exception ex)
            {
                // Halt and print rather than recover; the core's own autosave covers SRAM.
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
                RequestClose();
            }
        }

        // Spins rather than sleeps, for the reason measured in EmuSen_Settings_Reference.md §4.21.
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

        // Consumes the request flags once per frame; true means close - see EmuSen_Frontend_Driver.md §2.
        private bool ProcessHotkeys()
        {
            // Guarded on IsInputRedirected: KeyAvailable throws with no real console - see `man feed`.
            if (_feedWatchActive && !Console.IsInputRedirected && Console.KeyAvailable)
            {
                ConsoleKeyInfo key = Console.ReadKey(intercept: true);
                if ((key.Modifiers & ConsoleModifiers.Control) != 0 && key.Key == ConsoleKey.C)
                {
                    Console.WriteLine();
                    if (RunDebugPrompt()) return true;
                }
            }

            // A ROM dropped on the window, taken on this thread because SwapCore's is this one.
            if (_droppedRom is { } dropped)
            {
                _droppedRom = null;
                SwapCore(dropped);
            }

            // Every frame, cheap no-op when not recording.
            _frameRecorder.CaptureFrame(path => FrameImageWriter.SavePng(_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight, path));

            if (_requestToggleRecording) { _requestToggleRecording = false; ToggleRecording(); }
            if (_requestStartBgScrollTrace) { _requestStartBgScrollTrace = false; StartBgScrollTrace(); }
            if (_requestDumpBackdrop) { _requestDumpBackdrop = false; Venus?.Renderer!.DumpBackdropAndWindowDebugInfo(Venus.Bus!.Ppu, _core.TotalFrames); }
            if (_requestMirrorToggle) { _requestMirrorToggle = false; ToggleMirror(); }
            if (_requestSummary) { _requestSummary = false; Console.WriteLine(_debugTarget.GetSummaryText()); }
            if (_requestDumpOam) { _requestDumpOam = false; Venus?.Renderer!.DumpActiveOam(Venus.Bus!.Ppu); }
            if (_requestDebugPrompt) { _requestDebugPrompt = false; if (RunDebugPrompt()) return true; }
            if (_requestScreenshot) { _requestScreenshot = false; TakeScreenshot(); }
            if (_requestSaveState) { _requestSaveState = false; SaveState(_statePath); }
            if (_requestLoadState) { _requestLoadState = false; LoadState(_statePath); }

            if (_bgScrollTrace.ShouldLog())
            {
                if (Venus is { Bus: { } scrollBus }) Console.WriteLine($"[BG SCROLL] Frame {_core.TotalFrames}: BG1 X={scrollBus.Ppu.BgScrollX[0]} Y={scrollBus.Ppu.BgScrollY[0]}  |  BG2 X={scrollBus.Ppu.BgScrollX[1]} Y={scrollBus.Ppu.BgScrollY[1]}");
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

        // Blocks the emulation thread, never the UI thread - see EmuSen_Frontend_Driver.md §2.
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
                        // An empty line is not really a command, so it stays pre-dispatch.
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
                    // Pinned before the call: a `tmux switch` inside this Submit moves _scheduler - see `man tmux`.
                    DianaOSInterpreterScheduler haltedScheduler = _scheduler;
                    (_, string output, HostAction? action) = haltedScheduler.SubmitLocked(_debugCmd, trimmed);
                    Console.WriteLine(output);
                    if (action is HostAction.Shutdown) return true;
                    if (action is HostAction.SwitchSession)
                    {
                        // Halted is per-scheduler, so the old session would stay stuck - see `man tmux`.
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
            _frames.Offer(new FrameData { Rgba = rgba, Width = width, Height = height });
        }

        private void PresentPendingFrame(FrameData frame) =>
            GameFrame.UpdateFrame(frame.Rgba, frame.Width, frame.Height);

        private void Shutdown()
        {
            _gamepadTimer.Stop();
            _running = false;

            // Bounded: this thread may be blocked at a prompt - see EmuSen_Frontend_Driver.md §3f.
            _emuThread.Join(TimeSpan.FromMilliseconds(500));

            // The reader thread is not joined at all - see EmuSen_Frontend_Driver.md §3f.

            _core.SaveSram();
            _gamepad.Dispose();
            _audioPlayer.Dispose();
        }
    }
}
