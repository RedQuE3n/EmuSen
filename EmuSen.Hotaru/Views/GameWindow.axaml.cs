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
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Hotaru.Imaging;
using EmuSen.Hotaru.Input;
using EmuSen.Serenity;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Models;

namespace EmuSen.Hotaru.Views
{
    // Hotaru's game window and its three threads, over a session of the frontend API - see EmuSen_Frontend_Driver.md §1, §2 and §3f, and EmuSen_Backend.md §14.11.
    public partial class GameWindow : ToolWindow, ILiveShell, IHotaruHost
    {
        private readonly CrystalRuntime _runtime;

        // Replaced by a swap on the UI thread while the emulation thread waits, so it is read once per use.
        private volatile CrystalSession _session;
        private string _coreName;

        // The pointer gets out of the way over the game - see EmuSen_Frontend_Driver.md §3d.
        private IdleCursor? _idleCursor;
        private FileDrop? _fileDrop;

        // Set by a drop on the UI thread, taken by ProcessHotkeys, which asks the UI thread to swap. See §3d.
        private volatile string? _droppedRom;

        // See `man tmux`.
        private readonly DianaShSessionManager _sessions = new();

        // One scheduler per session - see `man tmux`.
        private readonly ConcurrentDictionary<DianaShSession, DianaShInterpreterScheduler> _schedulers = new();
        private DianaShInterpreter _debugCmd => _sessions.Current!.Interpreter;
        private DianaShInterpreterScheduler _scheduler => _schedulers[_sessions.Current!];

        // Reused unchanged across every rebuild - see EmuSen_Frontend_Driver.md §3f.
        private readonly IEnumerable<IDianaShCommand> _extraCommands;

        private readonly GamepadBindings _gamepadBindings;
        private readonly GamepadManager _gamepad;
        // Endymion is a leaf and reads no globals, so audio.json's settings come from here - see EmuSen_Audio_Sync.md §7.1.
        private readonly AudioPlayer _audioPlayer = NewAudioPlayer(AudioConfig.Load().Bounded());
        private readonly DispatcherTimer _gamepadTimer;

        // Tracked separately and OR'd: either device works at any time.
        private readonly bool[] _keyboardHeld = new bool[Enum.GetValues<PadControl>().Length];
        private readonly PortRouter _ports;
        private bool _mirrorPlayer1ToPlayer2;

        private volatile bool _running = true;
        private readonly Thread _emuThread;

        // Runs for the window's whole life, not just while halted - see EmuSen_Frontend_Driver.md §1.
        private readonly Thread _consoleReaderThread;

        // One writer (OnKeyDown), one reader/clearer (EmulationLoop) - see EmuSen_Frontend_Driver.md §2.
        private volatile bool _requestSummary;
        private volatile bool _requestScreenshot;
        private volatile bool _requestDebugPrompt;
        private volatile bool _requestSaveState;
        private volatile bool _requestLoadState;
        private volatile bool _requestToggleRecording;
        private volatile bool _requestMirrorToggle;

        // Held, not edge-triggered - see EmuSen_Rewind_And_FastForward.md §4.
        private volatile bool _turboHeld;
        private volatile bool _rewindHeld;
        private bool _turboApplied;
        private bool _recording;

        private readonly HashSet<Key> _heldPhysicalKeys = new();

        // 'feed's way back into the prompt without window focus - see `man feed`.
        private bool _feedWatchActive;

        // The last frame's facts, the picture's shape read by the feed window too.
        private CrystalFrameInfo _info;
        private volatile int _pictureWidth, _pictureHeight;
        private short[] _sound = new short[8192];

        // The session's state and session events, for every session this window opens - see EmuSen_Backend.md §14.11.
        private readonly HotaruEvents _events;
        private readonly Action _resetRateControl;

        // Pictures to the frame control in buffers made once, so a frame allocates none (C4) - see EmuSen_Backend.md §14.11.
        private readonly PictureSlots _pictures;

        private static AudioPlayer NewAudioPlayer(AudioConfig audio) => new(audio.SampleRate, audio.OutputTargetLatencyMs, audio.RateControlMaxDeviation);

        public GameWindow(HotaruSession session)
        {
            _pictures = new PictureSlots((pixels, width, height, release) => GameFrame.UpdateFrame(pixels, width, height, release: release));
            InitializeComponent();

            _runtime = session.Runtime;
            _session = session.Session;
            _coreName = session.CoreName;
            _events = new HotaruEvents(_runtime);
            _resetRateControl = () => _audioPlayer.RateControl.Reset();
            _extraCommands = HotaruCommands.For(this);
            _gamepadBindings = GamepadBindings.Load(HotaruSystems.ConsolesInReleaseOrder(_runtime));

            RebuildShells();
            // First load only; a swap deliberately does not re-add them - see EmuSen_Frontend_Driver.md §3f.
            AddFirstWatches();

            // graphics.json's window and picture; the title is the window's own - see EmuSen_Settings_Reference.md §3.
            GraphicsConfig graphics = GraphicsConfig.Load();
            (int width, int height) = graphics.WindowSize();
            Width = width;
            Height = height;
            CanResize = graphics.WindowResizable;
            GameFrame.BilinearFiltering = graphics.BilinearFiltering;
            GameFrame.SyncToDisplay = graphics.SyncToDisplay;

            HotaruControls controls = HotaruControls.Of(_session);
            _gamepad = new GamepadManager(_gamepadBindings.For(_coreName))
            {
                LeftStickIsAnalog = controls.Axes.Contains(PadAxis.LeftX),
                PlayerBindings = player => _gamepadBindings.For(_coreName, player),
                // Mistress's defaults, as Hotaru has no settings - see EmuSen_Input.md §8.13.
                ClaimPlayersByPress = true,
                ExternalControllerFirst = true,
            };
            // The keyboard is player 1's here; Hotaru has no setting to move it - see EmuSen_Input.md §8.5.
            _ports = new PortRouter(_gamepad, (p, b, held) => _session.Button(p, b, held), (p, a, v) => _session.Axis(p, a, v), (p, on) => _session.Connected(p, on))
            {
                KeyboardHeld = c => _keyboardHeld[(int)c],
            };
            _ports.Reset(controls.Ports, controls.Axes);
            _gamepad.BeginClaims();
            _gamepadTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1.0 / 60.0) };
            _gamepadTimer.Tick += (_, _) => PollGamepad();
            _gamepadTimer.Start();

            // Rewind kept by the runtime, gated by the engine inside the frame - see EmuSen_Backend.md §14.4.
            _session.Rewind(true);

            KeyDown += OnKeyDown;
            KeyUp += OnKeyUp;

            // The whole window, because the whole window is the game here.
            _idleCursor = new IdleCursor(this);
            // Queued rather than loaded: a swap is asked of the UI thread by the emulation thread between frames.
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

        // IHotaruHost: the session the window runs, for the commands made over it.
        CrystalSession IHotaruHost.Session => _session;

        CrystalRuntime IHotaruHost.Runtime => _runtime;

        private void AddFirstWatches()
        {
            try
            {
                _session.WatchAdd("WRAM", 0x8000, 0x1800, 0);
                _session.WatchAdd("WRAM", 0x0D80, 0x0080, 0);
            }
            catch (CrystalException) { }
        }

        // Sole owner of ConsoleLineReader.ReadLine - see EmuSen_Frontend_Driver.md §3f.
        private void ConsoleReaderLoop()
        {
            while (_running)
            {
                // Once per line, never re-read mid-line - see EmuSen_Frontend_Driver.md §3f.
                DianaShInterpreter shell = _debugCmd;

                if (!_scheduler.Halted) Console.Write("DianaSh $ ");

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
                if (action is HostAction.LoadCore loadCore) { Swap(HotaruLaunch.Rom(loadCore.RomPath)); continue; }
                if (action is HostAction.StartConsole start) { Swap(HotaruLaunch.Empty(start.Console)); continue; }
                if (action is HostAction.SwitchSession) SyncSchedulersToSessions();
                // Resume and Step came from the session's run events, and the session already runs.
            }
            return false;
        }

        private void OnKeyDown(object? sender, KeyEventArgs e)
        {
            if (!_heldPhysicalKeys.Add(e.Key)) return; // OS key-repeat, not a fresh press

            if (HotaruKeyMap.TryGetControl(e.Key, out PadControl control))
            {
                _keyboardHeld[(int)control] = true;
                ApplyKeys();
                return;
            }

            // Through the table, so the help window and the dispatch cannot disagree - see §3e.
            if (!HotaruHotkeys.TryGetAction(e.Key, out HotaruHotkey action)) return;

            switch (action)
            {
                case HotaruHotkey.Summary: _requestSummary = true; break;
                case HotaruHotkey.Screenshot: _requestScreenshot = true; break;
                case HotaruHotkey.DebugPrompt: _requestDebugPrompt = true; break;
                case HotaruHotkey.SaveState: _requestSaveState = true; break;
                case HotaruHotkey.ToggleRecording: _requestToggleRecording = true; break;
                case HotaruHotkey.MirrorToggle: _requestMirrorToggle = true; break;
                // Both are UI-side only, so they act here rather than being queued.
                case HotaruHotkey.ShowHotkeys: DebugWindows.ShowHotkeyHelpWindow(); break;
                case HotaruHotkey.CycleShader: CycleShaderEffect(); break; // pure UI-side (GameFrame.ActiveEffect) - no core state touched, safe to act on immediately
                case HotaruHotkey.LoadState: _requestLoadState = true; break;
                case HotaruHotkey.FastForward: _turboHeld = true; e.Handled = true; break; // Handled or Avalonia steals Tab for focus traversal
                case HotaruHotkey.Rewind: _rewindHeld = true; break;
            }
        }

        private void OnKeyUp(object? sender, KeyEventArgs e)
        {
            _heldPhysicalKeys.Remove(e.Key);
            if (HotaruKeyMap.TryGetControl(e.Key, out PadControl control))
            {
                _keyboardHeld[(int)control] = false;
                ApplyKeys();
                return;
            }

            if (!HotaruHotkeys.TryGetAction(e.Key, out HotaruHotkey action)) return;

            switch (action)
            {
                case HotaruHotkey.FastForward: _turboHeld = false; break;
                case HotaruHotkey.Rewind: _rewindHeld = false; break;
            }
        }

        // Through the API's input, not the bus; each player's pad to its own port, the keyboard to player 1's - see EmuSen_Input.md §8.5.
        private void ApplyKeys()
        {
            _ports.MirrorPlayer1ToPlayer2 = _mirrorPlayer1ToPlayer2;
            _ports.KeysChanged();
        }

        private void PollGamepad()
        {
            _gamepad.Poll();
            _ports.MirrorPlayer1ToPlayer2 = _mirrorPlayer1ToPlayer2;
            _ports.PollPads();
        }

        // Reuses FramePresenter's static NextEffect, not FramePresenter itself - see EmuSen_Serenity.md §4.
        private void CycleShaderEffect()
        {
            ShaderEffect effect = FramePresenter.NextEffect(GameFrame.ActiveEffect);
            GameFrame.ActiveEffect = effect;
            Console.WriteLine($"[SHADER] Active effect: {effect}");
        }

        // Every shell over the session's service table, rebuilt when a swap opens another session - see `man core` and `man tmux`.
        private void RebuildShells()
        {
            if (_sessions.Sessions.Count == 0)
            {
                var initial = new DianaShSession("main", BuildInterpreter());
                _sessions.RegisterInitial(initial);
                _schedulers[initial] = new DianaShInterpreterScheduler();
                return;
            }

            _sessions.RebuildAll(BuildInterpreter);
            SyncSchedulersToSessions();
        }

        private DianaShInterpreter BuildInterpreter() => DianaShInterpreter.CreateForService(_session.ServiceTable(), _extraCommands, _sessions);

        // Syncs _schedulers to whatever sessions currently exist - see `man tmux`.
        private void SyncSchedulersToSessions()
        {
            var live = new HashSet<DianaShSession>(_sessions.Sessions);
            foreach (DianaShSession stale in _schedulers.Keys.Where(s => !live.Contains(s)).ToList())
            {
                _schedulers.TryRemove(stale, out _);
            }
            foreach (DianaShSession session in _sessions.Sessions)
            {
                if (!_schedulers.ContainsKey(session)) _schedulers[session] = new DianaShInterpreterScheduler();
            }
        }

        // The emulation thread's: another game or an empty console, opened and the old session closed on the UI thread while this thread waits (C6) - see EmuSen_Backend.md §14.11.
        private void Swap(HotaruLaunch launch)
        {
            if (_recording)
            {
                Console.WriteLine("[CORE] Refusing to swap ROM while a recording is in progress - stop it first (F6).");
                return;
            }
            string? failed = Dispatcher.UIThread.Invoke(() => SwapOnUiThread(launch));
            if (failed is not null)
            {
                Console.WriteLine($"[CORE] {failed}");
                return;
            }
            _info = default;
            _audioPlayer.RateControl.Reset();
            Console.WriteLine(launch.RomPath is { } rom ? $"[CORE] Loaded: {rom}" : $"[CORE] Started: {launch}");
        }

        // The UI thread's: the new session first, so a game that will not open leaves the old one running; then the pads and every shell rebound here.
        internal string? SwapOnUiThread(HotaruLaunch launch)
        {
            CrystalSession opened;
            try { opened = HotaruSession.Open(_runtime, launch); }
            catch (CrystalException e) { return e.Message; }
            CrystalSession old = _session;
            _session = opened;
            _coreName = HotaruSession.CoreNameOf(opened);
            opened.Rewind(true);
            old.Dispose();

            HotaruControls controls = HotaruControls.Of(opened);
            // The new console's pad, not the outgoing one's - see EmuSen_Input.md §5.1.
            _gamepad.Bindings = _gamepadBindings.For(_coreName);
            _gamepad.LeftStickIsAnalog = controls.Axes.Contains(PadAxis.LeftX);
            _ports.Reset(controls.Ports, controls.Axes);
            _gamepad.BeginClaims();
            RebuildShells();
            return null;
        }

        // Runs entirely off the UI thread - see EmuSen_Frontend_Driver.md §1.
        private void EmulationLoop()
        {
            Stopwatch clock = Stopwatch.StartNew();
            TimeSpan nextTick = clock.Elapsed;

            // Shared with Mistress - see EmuSen_Settings_Reference.md §4.87.
            var scheduler = new FrameScheduler(GameFrame.Display.Current, Stopwatch.GetTimestamp());
            double frameRate = HotaruControls.FrameRateOf(_session);

            try
            {
                while (_running)
                {
                    CrystalSession session = _session;
                    if (_turboHeld != _turboApplied)
                    {
                        _turboApplied = _turboHeld;
                        session.SpeedMode(_turboApplied ? CrystalAbi.SpeedTurbo : CrystalAbi.SpeedNormal);
                    }
                    if (_info.frame_rate_hz > 0) frameRate = _info.frame_rate_hz;
                    TimeSpan interval = _info.frame_interval_ns > 0 ? TimeSpan.FromTicks((long)(_info.frame_interval_ns / 100)) : TimeSpan.FromSeconds(1.0 / frameRate);
                    scheduler.SyncToDisplay = GameFrame.SyncToDisplay;
                    nextTick = scheduler.Next(nextTick, frameRate, !_turboApplied, interval);

                    // Takes over the frame entirely - see EmuSen_Rewind_And_FastForward.md §4.
                    if (_rewindHeld)
                    {
                        if (session.RewindStep(1, ref _info) == 0) nextTick = clock.Elapsed;
                        DrainSound(session, play: false);
                        SubmitPicture(session);
                        if (DrainEvents()) { RequestClose(); return; }
                        if (ProcessPendingConsoleCommands()) { RequestClose(); return; }
                        SleepUntil(nextTick, clock);
                        continue;
                    }

                    long frameStart = Stopwatch.GetTimestamp();
                    session.Frame(ref _info);

                    // A breakpoint or armed step stopped the frame; the session holds itself halted until run control resumes it.
                    if (_info.halted != 0)
                    {
                        Console.WriteLine($"\n[BREAKPOINT] Halted at ${unchecked((int)(uint)_info.halted_address):X6}");
                        Console.WriteLine(session.Summary());
                        if (RunDebugPrompt()) { RequestClose(); return; }
                        nextTick = clock.Elapsed; // don't burst-catch-up for time spent at the prompt
                        continue;
                    }

                    // Drained every frame either way, so a muted stretch cannot back the buffer up - see EmuSen_Audio_Sync.md §4.
                    _audioPlayer.RateControl.NominalRatio = scheduler.Decision.AudioRatio;
                    DrainSound(session, play: _info.play_audio != 0);

                    // Nothing new was drawn on a skipped frame.
                    long held = 0;
                    if (scheduler.HandOverNotBefore is { } notBefore && clock.Elapsed < notBefore)
                    {
                        long holdStart = Stopwatch.GetTimestamp();
                        SleepUntil(notBefore, clock);
                        held = Stopwatch.GetTimestamp() - holdStart;
                    }
                    if (_info.rendered != 0) SubmitPicture(session);
                    scheduler.Completed(TimeSpan.FromTicks((Stopwatch.GetTimestamp() - frameStart - held) * TimeSpan.TicksPerSecond / Stopwatch.Frequency));

                    if (DrainEvents()) { RequestClose(); return; }
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
                // Halt and print rather than recover; closing the session writes the battery save.
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
                RequestClose();
            }
        }

        // The sound waiting, played or dropped as the speed says; a dropped stretch resets the rate control - see EmuSen_Audio_Sync.md §3.2.
        private void DrainSound(CrystalSession session, bool play)
        {
            int frames = session.AudioFramesReady;
            if (_sound.Length < frames * 2) _sound = new short[frames * 2];
            int got = session.DrainAudio(_sound, out uint rate);
            if (!play || got == 0)
            {
                if (!play) _audioPlayer.RateControl.Reset();
                return;
            }
            _audioPlayer.Submit(_sound.AsSpan(0, got * 2).ToArray(), (int)rate);
        }

        // The picture into a buffer the hand-off owns; none free means the screen still holds them all, and this one is skipped.
        private void SubmitPicture(CrystalSession session)
        {
            int width = _info.width, height = _info.height;
            if (width <= 0 || height <= 0) return;
            _pictureWidth = width;
            _pictureHeight = height;
            if (_pictures.Rent(width * height * 4) is not { } pixels) return;
            session.CopyPicture(pixels);
            _pictures.Offer(width, height);
        }

        // A state loaded resets the audio rate control, whoever loaded it (C7); true when the session was closed under the window.
        private bool DrainEvents() => _events.Drain(_session.Id, _resetRateControl);

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

            // A ROM dropped on the window, swapped between frames as `core` swaps one.
            if (_droppedRom is { } dropped)
            {
                _droppedRom = null;
                Swap(HotaruLaunch.Rom(dropped));
            }

            if (_requestToggleRecording) { _requestToggleRecording = false; ToggleRecording(); }
            if (_requestMirrorToggle) { _requestMirrorToggle = false; ToggleMirror(); }
            if (_requestSummary) { _requestSummary = false; Console.WriteLine(_session.Summary()); }
            if (_requestDebugPrompt) { _requestDebugPrompt = false; if (RunDebugPrompt()) return true; }
            if (_requestScreenshot) { _requestScreenshot = false; TakeScreenshot(); }
            if (_requestSaveState) { _requestSaveState = false; SaveState(null); }
            if (_requestLoadState) { _requestLoadState = false; LoadState(null); }

            return false;
        }

        // The picture now, in an array of its own: for a screenshot, and for the feed window on the UI thread (C3).
        internal (byte[] Rgba, int Width, int Height) CopyPicture()
        {
            int width = _pictureWidth, height = _pictureHeight;
            if (width <= 0 || height <= 0) return (Array.Empty<byte>(), 0, 0);
            var rgba = new byte[width * height * 4];
            try { _session.CopyPicture(rgba); }
            catch (CrystalException) { return (Array.Empty<byte>(), 0, 0); }
            return (rgba, width, height);
        }

        private void TakeScreenshot()
        {
            string coreLogDir = Path.Combine(DianaShSandbox.LogsDirectory, _coreName);
            Directory.CreateDirectory(coreLogDir);
            ulong frame = _info.total_frames;
            string baseName = $"screenshot_frame{frame}";
            string shotPath = Path.Combine(coreLogDir, baseName + ".png");
            string metaPath = Path.Combine(coreLogDir, baseName + ".txt");

            var (rgba, width, height) = CopyPicture();
            FrameImageWriter.SavePng(rgba, width, height, shotPath);
            File.WriteAllText(metaPath,
                $"Core: {_coreName}\n" +
                $"Frame: {frame}\n" +
                $"Wall-clock: {DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}\n");

            Console.WriteLine($"[SCREENSHOT] Saved {shotPath} (Core={_coreName} Frame={frame})");
        }

        // The runtime's state_save; with no path, the player's state folder's.
        private void SaveState(string? path)
        {
            try
            {
                _session.SaveState(path);
                Console.WriteLine($"[STATE] Saved: {path ?? _session.DefaultStatePath()}");
            }
            catch (CrystalException ex)
            {
                Console.WriteLine($"[STATE] Save failed: {ex.Message}");
            }
        }

        // The runtime's state_load clears the rewind history and raises the event that resets the rate control (C7).
        private void LoadState(string? path)
        {
            try
            {
                _session.LoadState(path);
                Console.WriteLine($"[STATE] Loaded: {path ?? _session.DefaultStatePath()}");
            }
            catch (CrystalException ex)
            {
                Console.WriteLine($"[STATE] Load failed: {ex.Message}");
            }
        }

        private void ToggleRecording()
        {
            try
            {
                if (_recording)
                {
                    Console.WriteLine($"[RECORD] Stopped: {_session.StopRecording()}");
                    _recording = false;
                }
                else
                {
                    string dir = _session.StartRecording(Path.Combine(DianaShSandbox.LogsDirectory, _coreName, "Recordings"), 1);
                    _recording = true;
                    Console.WriteLine($"[RECORD] Started -> {dir}");
                }
            }
            catch (CrystalException ex)
            {
                Console.WriteLine($"[RECORD] {ex.Message}");
            }
        }

        private void ToggleMirror()
        {
            _mirrorPlayer1ToPlayer2 = !_mirrorPlayer1ToPlayer2;
            Console.WriteLine($"[INPUT] Mirror Player 1 -> Player 2: {(_mirrorPlayer1ToPlayer2 ? "ON" : "OFF")}");
        }

        // Blocks the emulation thread, never the UI thread; leaving it resumes the session unless a swap or a step ran it - see EmuSen_Frontend_Driver.md §2.
        private bool RunDebugPrompt()
        {
            DisarmFeedWatch();
            _scheduler.Halted = true;
            try
            {
                Console.WriteLine("--- DianaSh (type 'help', 'resume' to resume, 'feed'/'feed -w' to resume and watch gameplay, 'shutdown' to quit, 'step'/'s' to single-step, 'core <name> <path>' to swap ROMs) ---");
                while (true)
                {
                    Console.Write(_debugCmd.IsAwaitingMoreInput ? "> " : "DianaSh #: ");
                    string line = _scheduler.TakeHaltedLine();
                    string trimmed = line.Trim();

                    if (!_debugCmd.IsAwaitingMoreInput)
                    {
                        // An empty line is not really a command, so it stays pre-dispatch.
                        if (trimmed.Length == 0)
                        {
                            Resume();
                            break;
                        }
                        if (trimmed.Equals("feed", StringComparison.OrdinalIgnoreCase) || trimmed.StartsWith("feed ", StringComparison.OrdinalIgnoreCase))
                        {
                            string[] feedParts = trimmed.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                            bool windowed = feedParts.Length >= 2 && feedParts[1].Equals("-w", StringComparison.OrdinalIgnoreCase);
                            if (windowed)
                            {
                                DebugWindows.ShowFeedWindow(CopyPicture);
                                Console.WriteLine("[FEED] Opened in a separate window.");
                            }
                            ArmFeedWatch();
                            Resume();
                            Console.WriteLine("Resuming - press Ctrl+C in this terminal to reopen the prompt.");
                            break;
                        }
                    }
                    // Pinned before the call: a `tmux switch` inside this Submit moves _scheduler - see `man tmux`.
                    DianaShInterpreterScheduler haltedScheduler = _scheduler;
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
                    if (action is HostAction.LoadCore loadCore) { Swap(HotaruLaunch.Rom(loadCore.RomPath)); break; }
                    if (action is HostAction.StartConsole start) { Swap(HotaruLaunch.Empty(start.Console)); break; }
                }
                Console.WriteLine("--- Resuming ---");
                return false;
            }
            finally
            {
                _scheduler.Halted = false;
            }
        }

        // Leaving the prompt with no resume of its own: the session runs again, a halt released.
        private void Resume()
        {
            try { _session.Run(CrystalSessionAbi.RunResume); }
            catch (CrystalException ex) { Console.WriteLine($"[CORE] {ex.Message}"); }
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

        private void Shutdown()
        {
            _gamepadTimer.Stop();
            _running = false;

            // Bounded: this thread may be blocked at a prompt - see EmuSen_Frontend_Driver.md §3f.
            _emuThread.Join(TimeSpan.FromMilliseconds(500));

            // The reader thread is not joined at all - see EmuSen_Frontend_Driver.md §3f.

            // Closing the session writes the battery save, as SaveSram did.
            _events.Dispose();
            _session.Dispose();
            _gamepad.Dispose();
            _audioPlayer.Dispose();
        }
    }
}
