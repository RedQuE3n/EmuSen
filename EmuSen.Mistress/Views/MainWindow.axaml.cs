using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Mistress.Audio;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Settings;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Mistress.Views
{
    public partial class MainWindow : Window
    {
        // ~60fps. Not synced to the core's actual scanline timing yet - this
        // is a fixed-interval pacing target, which is fine for a first pass
        // but will drift from real SNES frame timing over long sessions.
        // Worth revisiting (e.g. accumulator-based stepping) once this is
        // otherwise working.
        private static readonly TimeSpan FrameInterval = TimeSpan.FromSeconds(1.0 / 60.0);

        private EmulatorSession? _session;

        // Gamepad polling stays on the UI thread, on its own timer, separate
        // from emulation itself (see _emuThread below) - GamepadManager.cs's
        // own comment flags Silk.NET.SDL's exact API surface as unverified,
        // and SDL's threading rules are platform-specific enough (some
        // platforms expect event/controller polling to stay on the thread
        // that called SDL_Init, which is this constructor's thread, i.e. the
        // UI thread) that keeping it here is the safe default rather than
        // something worth risking on a perf pass.
        private DispatcherTimer? _timer;

        // Runs RunFrame() + frame-buffer readout off the UI thread entirely -
        // previously both lived inside the DispatcherTimer tick above, which
        // meant emulation work (and the WriteableBitmap copy after it)
        // blocked Avalonia's UI/input/paint pump every single frame. This is
        // the actual fix for Testing Studio running much slower than the
        // console build, which always ran its equivalent loop off of any UI
        // event pump to begin with (it doesn't have one). See
        // EmulationLoop()/StartEmulationThread()/StopEmulationThread() below.
        private Thread? _emuThread;
        private volatile bool _running;

        // Pause/resume for the emulation thread, added so a shell command
        // typed into _consoleWindow can safely read/write Cpu/Bus/Renderer
        // state without racing RunFrame() on _emuThread - see
        // EmulationControlCommands.cs's own comment for why this exists.
        // Signaled (Set) = running, unsignaled (Reset) = paused; starts
        // signaled so a freshly-started thread doesn't block before
        // anyone's had a chance to pause it. EmulationLoop blocks on this
        // at the top of every iteration rather than busy-polling a bool,
        // and StopEmulationThread always Sets it before Join()ing so a
        // paused thread can still wake up, notice _running is false, and
        // exit - otherwise stopping while paused would deadlock forever.
        private readonly ManualResetEventSlim _pauseSignal = new(initialState: true);

        public bool IsPaused => !_pauseSignal.IsSet;

        // Coalescing hand-off from _emuThread to the UI thread: the
        // emulation thread can produce frames faster than Avalonia can
        // present them, and posting one Dispatcher action per emulated frame
        // would just queue them up and make the UI thread fall further and
        // further behind. Instead, only ever have at most one Present
        // dispatched at a time - _emuThread always overwrites _pendingFrame
        // with the newest frame and only schedules a new Present if one
        // isn't already in flight, so a slow UI thread simply drops
        // intermediate frames rather than backing up a queue of them.
        private sealed class FrameData
        {
            public required byte[] Pixels;
            public required int Width;
            public required int Height;
        }
        private FrameData? _pendingFrame;
        private int _presentScheduled;

        // Rebuilt on every LoadRom() call (see LoadRom below) so a shell
        // command run in _consoleWindow always sees whatever core is
        // actually running now, never a Cpu/Bus/Renderer left over from a
        // ROM that's since been swapped out. Null before the first ROM
        // loads - the console window (and every shell command's own
        // RequireTarget guard) already treats that as a normal condition.
        private SnesDebugTarget? _debugTarget;

        // At most one console window at a time - Show()n non-modally (same
        // pattern DebugSettingsWindow/InputSettingsWindow already use), and
        // reused (brought to front) rather than duplicated if the menu item
        // is clicked again while one's still open. Cleared on Closed so a
        // later LoadRom() doesn't try to push a target update into a
        // disposed window.
        private DianaOSConsoleWindow? _consoleWindow;

        // Same at-most-one/reuse/clear-on-Closed pattern as _consoleWindow
        // above - opened from inside the console window itself (typing
        // `coretop`, via CoretopWindowCommand), not from a menu item.
        private CoretopWindow? _coretopWindow;

        private string? _currentRomPath;
        private string? _currentDisplayName; // for restoring StatusText's "Running: ..." text exactly after a pause, without reformatting from _currentRomPath
        private readonly ControllerKeyMap _keyBindings = ControllerKeyMap.Load();
        private readonly GamepadBindingMap _gamepadBindings = GamepadBindingMap.Load();
        private readonly AppSettings _appSettings = AppSettings.Load();
        private readonly GamepadManager _gamepad;

        // Constructed once at startup and reused across every ROM load,
        // same lifetime as _gamepad above (and for the same reason -
        // Pump() below just no-ops via EmulatorSession's own null-session
        // fallback when nothing's loaded, so there's no need to
        // open/close the output device per load).
        private readonly AudioPlayer _audioPlayer;

        // Captured once at startup, before anything ever redirects
        // Console.Out - restoring this (rather than whatever
        // _activeLogWriter happened to be at the time) is what lets logging
        // be turned off again (or repointed) without leaving Console.Out
        // pointed at a disposed writer.
        private readonly TextWriter _originalConsoleOut = Console.Out;
        private CategorizedLogWriter? _activeLogWriter;

        // Keyboard and gamepad are tracked separately and combined with OR
        // logic - matches EmuSen.Hotaru's own GameWindow convention
        // ("either device works at any time, no need to pick
        // one"). Without this, a gamepad poll finding a button NOT pressed
        // would incorrectly release a button still being held on the
        // keyboard, and vice versa.
        private readonly bool[] _keyboardHeld = new bool[Enum.GetValues<SnesButton>().Length];
        private readonly bool[] _gamepadHeld = new bool[Enum.GetValues<SnesButton>().Length];

        public MainWindow()
        {
            InitializeComponent();
            _gamepad = new GamepadManager(_gamepadBindings);
            _audioPlayer = new AudioPlayer();
            Closing += (_, _) =>
            {
                _timer?.Stop();
                StopEmulationThread();
                _session?.SaveSram();
                _gamepad.Dispose();
                _audioPlayer.Dispose();
                StopLogging();
            };

            KeyDown += (_, e) => SetButtonFromKey(e.Key, pressed: true);
            KeyUp += (_, e) => SetButtonFromKey(e.Key, pressed: false);
        }

        private void SetButtonFromKey(Key key, bool pressed)
        {
            if (_keyBindings.TryGetButton(key, out var button))
            {
                _keyboardHeld[(int)button] = pressed;
                ApplyButtonState(button);
            }
        }

        private void ApplyButtonState(SnesButton button)
        {
            if (_session is not { IsRomLoaded: true }) return;
            bool held = _keyboardHeld[(int)button] || _gamepadHeld[(int)button];
            _session.Bus.Input.SetButton(button, held);
            if (_appSettings.MirrorPlayer1ToPlayer2) _session.Bus.Input.SetButton(button, held, controller: 2);
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

        private async void OnOpenRomClick(object? sender, RoutedEventArgs e)
        {
            IStorageFolder? startLocation = null;
            if (!string.IsNullOrWhiteSpace(_appSettings.RomDirectory) && Directory.Exists(_appSettings.RomDirectory))
            {
                // TryGetFolderFromPathAsync takes a Uri, not a plain path string.
                startLocation = await StorageProvider.TryGetFolderFromPathAsync(new Uri(_appSettings.RomDirectory));
            }

            var files = await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
            {
                Title = "Open ROM",
                AllowMultiple = false,
                SuggestedStartLocation = startLocation,
                FileTypeFilter = new[]
                {
                    new FilePickerFileType("SNES ROMs") { Patterns = new[] { "*.smc", "*.sfc" } },
                    new FilePickerFileType("All files") { Patterns = new[] { "*" } },
                }
            });

            var file = files.FirstOrDefault();
            if (file is null) return;

            LoadRom(file.Path.LocalPath, file.Name);
        }

        // Alternative to the OS file picker above - lists .smc/.sfc files
        // straight from AppSettings.RomDirectory, for anyone reloading
        // different ROMs from the same test folder repeatedly. See
        // RomBrowserWindow's own comment.
        private async void OnBrowseRomsClick(object? sender, RoutedEventArgs e)
        {
            string? selected = await new RomBrowserWindow(_appSettings.RomDirectory).ShowDialog<string?>(this);
            if (selected is null) return;

            LoadRom(selected, Path.GetFileName(selected));
        }

        private void OnControllerBindingsClick(object? sender, RoutedEventArgs e)
        {
            new InputSettingsWindow(_keyBindings, _gamepadBindings, _gamepad, _appSettings).Show(this);
        }

        private void OnPreferencesClick(object? sender, RoutedEventArgs e)
        {
            new PreferencesWindow(_appSettings).Show(this);
        }

        private void OnDebugLoggingClick(object? sender, RoutedEventArgs e)
        {
            new DebugSettingsWindow().Show(this);
        }

        private void OnShellConsoleClick(object? sender, RoutedEventArgs e)
        {
            if (_consoleWindow is not null)
            {
                _consoleWindow.Activate();
                return;
            }

            _consoleWindow = new DianaOSConsoleWindow(_debugTarget, MakeEmulationControlCommands());
            _consoleWindow.Closed += (_, _) => _consoleWindow = null;
            _consoleWindow.Show(this);
        }

        // Built fresh per DianaOSConsoleWindow construction (not cached) -
        // each PauseCommand/ResumeCommand instance only needs to close
        // over `this`, so there's no real cost to re-creating them, and
        // it avoids the two commands' delegates ever accidentally
        // outliving a MainWindow instance.
        private IEnumerable<IDianaOSCommand> MakeEmulationControlCommands() => new IDianaOSCommand[]
        {
            new PauseCommand(PauseEmulation, () => IsPaused),
            new ResumeCommand(ResumeEmulation, () => IsPaused),
            new CoretopWindowCommand(OpenCoretopWindow),
            new FeedCommand(() => Activate()),
            new EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.StateCommand(SaveStateFromConsole, LoadStateFromConsole, () => CurrentStatePath ?? ""),
        };

        // StateCommand's save/load delegates - can't just be
        // `_session.SaveState`/`LoadState` method groups the way
        // MakeEmulationControlCommands() reuses PauseEmulation/
        // ResumeEmulation directly, since `_session` can still be null the
        // first time this window's console is opened (before any ROM has
        // ever loaded) - a method-group conversion would dereference it
        // immediately, at command-construction time, not when the command
        // actually runs. These defer that check to call time instead, and
        // throw the same clean message CoretopWindowCommand's own
        // RequireTarget-backed "No ROM loaded" case already establishes,
        // rather than letting StateCommand's try/catch surface a bare
        // NullReferenceException's own unhelpful message.
        private void SaveStateFromConsole(string path)
        {
            if (_session is null) throw new InvalidOperationException("No ROM loaded.");
            _session.SaveState(path);
        }

        private void LoadStateFromConsole(string path)
        {
            if (_session is null) throw new InvalidOperationException("No ROM loaded.");
            _session.LoadState(path);
        }

        // Opens (or brings forward/updates) CoretopWindow - same at-most-
        // one/reuse pattern as OnShellConsoleClick uses for
        // _consoleWindow, just triggered from inside the console window
        // itself (typing `coretop`) rather than a menu item.
        private void OpenCoretopWindow(IDebugTarget? target)
        {
            if (_coretopWindow is not null)
            {
                _coretopWindow.UpdateTarget(target);
                _coretopWindow.Activate();
                return;
            }

            _coretopWindow = new CoretopWindow(target);
            _coretopWindow.Closed += (_, _) => _coretopWindow = null;
            _coretopWindow.Show(this);
        }

        private string? CurrentStatePath =>
            _currentRomPath is null
                ? null
                : System.IO.Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                    "EmuSen", "Saves",
                    System.IO.Path.GetFileNameWithoutExtension(_currentRomPath) + ".state");

        private void OnSaveStateClick(object? sender, RoutedEventArgs e)
        {
            if (_session is not { IsRomLoaded: true } || CurrentStatePath is not string path)
            {
                StatusText.Text = "Save State: no ROM loaded";
                return;
            }

            try
            {
                System.IO.Directory.CreateDirectory(System.IO.Path.GetDirectoryName(path)!);
                _session.SaveState(path);
                StatusText.Text = $"State saved: {System.IO.Path.GetFileName(path)}";
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Save State failed: {ex.Message}";
            }
        }

        private void OnLoadStateClick(object? sender, RoutedEventArgs e)
        {
            if (_session is not { IsRomLoaded: true } || CurrentStatePath is not string path)
            {
                StatusText.Text = "Load State: no ROM loaded";
                return;
            }

            if (!System.IO.File.Exists(path))
            {
                StatusText.Text = "Load State: no save state found for this ROM";
                return;
            }

            try
            {
                _session.LoadState(path);
                StatusText.Text = $"State loaded: {System.IO.Path.GetFileName(path)}";
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Load State failed: {ex.Message}";
            }
        }

        private void LoadRom(string path, string displayName)
        {
            _timer?.Stop();
            StopEmulationThread(); // must fully stop before _session is replaced below - see that method's own comment
            _session?.SaveSram(); // flush whatever was previously running before switching
            // Must happen on the OLD session, before it's replaced below -
            // StartLogging()'s call to StopLogging() runs against whatever
            // _session currently is, which would already be the new one.
            _session?.FlushVerboseLogs();

            try
            {
                _session = new EmulatorSession();
                StartLogging(_session.CoreName); // before LoadRom() so Cartridge's own load-time output is captured too
                _session.LoadRom(path);

                // See SnesDebugTarget's own constructor comment - feeds
                // `coretop`'s hardware-load bars.
                _debugTarget = new SnesDebugTarget(_session.Cpu!, _session.Bus, _session.Renderer!,
                    () => (_session.LastFrameCpuSpc700Ms, _session.LastFramePpuMs, _session.LastFrameHdmaMs));
                _consoleWindow?.UpdateTarget(_debugTarget, displayName);
                _coretopWindow?.UpdateTarget(_debugTarget);

                GameFrame.IsVisible = true;
                NoRomText.IsVisible = false;

                StatusText.Text = $"Running: {displayName}";
                _currentRomPath = path;
                _currentDisplayName = displayName;

                // Gamepad polling only - see _timer's own field comment for
                // why this stays separate from emulation itself.
                _timer = new DispatcherTimer { Interval = FrameInterval };
                _timer.Tick += (_, _) => PollGamepad();
                _timer.Start();

                StartEmulationThread();
            }
            catch (Exception ex)
            {
                _session = null;
                StatusText.Text = $"Failed to load {displayName}: {ex.Message}";
            }
        }

        // Called from _consoleWindow's PauseCommand/ResumeCommand (both run
        // on the UI thread, same as this method) - ManualResetEventSlim's
        // Set/Reset are thread-safe regardless, so there's nothing else to
        // guard here. StatusText is only ever touched from the UI thread
        // in either case, so no Dispatcher.UIThread.Post is needed the way
        // EmulationLoop needs one for its own cross-thread updates.
        public void PauseEmulation()
        {
            _pauseSignal.Reset();
            if (_session is { IsRomLoaded: true }) StatusText.Text = "Paused";
        }

        public void ResumeEmulation()
        {
            _pauseSignal.Set();
            if (_session is { IsRomLoaded: true }) StatusText.Text = $"Running: {_currentDisplayName}";
        }

        private void StartEmulationThread()
        {
            _running = true;
            _emuThread = new Thread(EmulationLoop) { IsBackground = true, Name = "EmuSen-Emulation" };
            _emuThread.Start();
        }

        // Signals the loop to stop and waits for it to actually exit before
        // returning - callers (LoadRom, window Closing) need this to be
        // synchronous since they go on to replace/dispose _session right
        // after. Safe to call from the UI thread: EmulationLoop checks
        // _running once per paced ~16ms frame interval at most, so Join()
        // here never blocks for long, and it never blocks indefinitely since
        // the loop has no other blocking wait.
        private void StopEmulationThread()
        {
            if (_emuThread is null) return;
            _running = false;
            _pauseSignal.Set(); // wake the thread if it's currently paused, so it can observe _running=false and exit rather than deadlocking Join() below
            _emuThread.Join();
            _emuThread = null;
        }

        // Runs entirely off the UI thread: RunFrame() (the actual CPU/PPU/
        // APU work) and the frame-buffer readout no longer compete with
        // Avalonia's input/paint pump the way they did inside the old
        // DispatcherTimer tick. Only the final present (PresentPendingFrame,
        // dispatched below) needs the UI thread - GameFrame (GameFrameControl)
        // is a regular Avalonia Control, and Control/Visual are UI-thread-
        // affine the same way WriteableBitmap used to be.
        //
        // Input note: keyboard/gamepad button state is applied straight to
        // _session.Bus.Input from the UI thread (SetButtonFromKey,
        // PollGamepad's ApplyButtonState calls) while this thread
        // concurrently calls RunFrame(), which reads that same state
        // (Input.cs's LatchAutoJoypad/live-state fields). Deliberately left
        // unsynchronized: each field involved is a single ushort/bool
        // read-modify-write with exactly one writer (the UI thread), so the
        // worst case is a button's state being observed one frame later
        // than it otherwise would - not a torn read or a crash - which is
        // an acceptable, already-inherent latency for a frame-latched input
        // model like this one, not a new correctness risk introduced by
        // moving emulation to its own thread.
        private void EmulationLoop()
        {
            Stopwatch clock = Stopwatch.StartNew();
            TimeSpan nextTick = clock.Elapsed;

            // Measured emulated FPS (TotalFrames delta / real elapsed time),
            // updated once a second - a diagnostic for telling "RunFrame()
            // itself is running behind real time" apart from "presentation
            // is just choppy but emulation is on schedule". Deliberately
            // counts completed RunFrame() calls, not presented frames -
            // SubmitFrame()/PresentPendingFrame's coalescing can legitimately
            // drop presented frames without that meaning emulation itself
            // is slow.
            TimeSpan fpsWindowStart = clock.Elapsed;
            int framesInWindow = 0;

            // Split timing: RunFrame() alone vs. the rest of this loop's own
            // per-frame work (GetFrameBufferRgba's copy + SubmitFrame's
            // hand-off). Boot/title screens hitting 60fps while gameplay
            // drops below it could mean either RunFrame() itself getting
            // more expensive under a heavier scene (shared core code -
            // would cost the console build the same) or this loop's own
            // wrapper overhead scaling up with scene complexity (frontend-
            // specific, e.g. more per-frame allocation/GC pressure) -
            // reporting both separately is how to tell which.
            TimeSpan runFrameTimeInWindow = TimeSpan.Zero;
            var frameStopwatch = new Stopwatch();

            // VenusCore's own per-scanline-granularity phase breakdown
            // (VenusCore.RunFrame()'s own comment) - averaged the same way
            // as runFrameTimeInWindow above, to see which subsystem
            // (CPU+SPC700 stepping, PPU rendering, HDMA) actually accounts
            // for RunFrame() itself getting slower during real gameplay.
            double cpuSpc700MsInWindow = 0;
            double ppuMsInWindow = 0;
            double hdmaMsInWindow = 0;
            double objEvalMsInWindow = 0;
            double blendMsInWindow = 0;
            double mainCompositeMsInWindow = 0;
            double subCompositeMsInWindow = 0;

            while (_running)
            {
                // Blocks here, not inside the try below, while paused -
                // see _pauseSignal's own field comment. StopEmulationThread
                // always Sets this before Join()ing, so this can never
                // block forever even if a console command pauses and the
                // window is then closed without resuming first. Checked
                // before waiting (rather than always calling Wait(), which
                // would also be correct but always costs a syscall even
                // when never paused) so the by-far-more-common unpaused
                // case stays a plain volatile-ish read.
                if (!_pauseSignal.IsSet)
                {
                    _pauseSignal.Wait();
                    if (!_running) break;

                    // Otherwise nextTick would still be wherever it was
                    // when the pause began, and the "fell behind" branch
                    // at the bottom of this loop would attribute the
                    // entire paused duration to a single artificially slow
                    // frame in the fps window above.
                    nextTick = clock.Elapsed;
                }

                nextTick += FrameInterval;

                EmulatorSession? session = _session;
                if (session is null) break;

                try
                {
                    frameStopwatch.Restart();
                    session.RunFrame();
                    runFrameTimeInWindow += frameStopwatch.Elapsed;

                    // Publishes this frame's register/sprite/palette/audio/
                    // hardware-load snapshots via IDebugTarget's provider
                    // properties - see EmuSen.Cauldron.IRealtimeProvider's
                    // own comment. Null-conditional since _debugTarget isn't
                    // atomically tied to _session (a ROM swap in flight
                    // could momentarily leave one set without the other).
                    _debugTarget?.RefreshProviders();

                    // Runs whatever _consoleWindow queued (a mutating
                    // command typed while not fast-path-eligible - see
                    // DianaOSConsoleWindow.Submit and
                    // EmuSen.DianaOS.DianaOS.Bin.DianaOSInterpreterScheduler's own
                    // comment) against the core we just finished a frame
                    // on - null-conditional since the console window is
                    // opened on demand and may not exist at all. Must run
                    // on this (the emulation) thread, same reasoning as
                    // RefreshProviders() just above.
                    _consoleWindow?.DrainPendingFromEmulationThread();

                    // Same call-site placement as PumpAudio() in
                    // EmuSen.Hotaru/Program.cs - right after
                    // RunFrame(), since that's what actually produces new
                    // samples to drain. Safe here on _emuThread rather than
                    // the UI thread - see AudioPlayer.Pump's own comment.
                    _audioPlayer.Pump(session);
                    cpuSpc700MsInWindow += session.LastFrameCpuSpc700Ms;
                    ppuMsInWindow += session.LastFramePpuMs;
                    hdmaMsInWindow += session.LastFrameHdmaMs;
                    objEvalMsInWindow += session.LastFrameObjEvalMs;
                    blendMsInWindow += session.LastFrameBlendMs;
                    mainCompositeMsInWindow += session.LastFrameMainCompositeMs;
                    subCompositeMsInWindow += session.LastFrameSubCompositeMs;

                    byte[] frame = session.GetFrameBufferRgba();
                    SubmitFrame(frame, session.ScreenWidth, EmulatorSession.ScreenHeight);

                    framesInWindow++;
                    TimeSpan windowElapsed = clock.Elapsed - fpsWindowStart;
                    if (windowElapsed >= TimeSpan.FromSeconds(1))
                    {
                        double fps = framesInWindow / windowElapsed.TotalSeconds;
                        double runFrameMs = runFrameTimeInWindow.TotalMilliseconds / framesInWindow;
                        double totalMs = windowElapsed.TotalMilliseconds / framesInWindow;
                        double cpuSpc700Ms = cpuSpc700MsInWindow / framesInWindow;
                        double ppuMs = ppuMsInWindow / framesInWindow;
                        double hdmaMs = hdmaMsInWindow / framesInWindow;
                        double objEvalMs = objEvalMsInWindow / framesInWindow;
                        double blendMs = blendMsInWindow / framesInWindow;
                        double mainCompositeMs = mainCompositeMsInWindow / framesInWindow;
                        double subCompositeMs = subCompositeMsInWindow / framesInWindow;
                        Dispatcher.UIThread.Post(() => FpsText.Text =
                            $"{fps:F1} fps (run {runFrameMs:F2}ms / total {totalMs:F2}ms) " +
                            $"[cpu+apu {cpuSpc700Ms:F2}ms / ppu {ppuMs:F2}ms / hdma {hdmaMs:F2}ms] " +
                            $"(ppu breakdown: objEval {objEvalMs:F2}ms / main {mainCompositeMs:F2}ms / " +
                            $"sub {subCompositeMs:F2}ms / blend {blendMs:F2}ms)");
                        framesInWindow = 0;
                        cpuSpc700MsInWindow = 0;
                        ppuMsInWindow = 0;
                        hdmaMsInWindow = 0;
                        objEvalMsInWindow = 0;
                        blendMsInWindow = 0;
                        mainCompositeMsInWindow = 0;
                        subCompositeMsInWindow = 0;
                        runFrameTimeInWindow = TimeSpan.Zero;
                        fpsWindowStart = clock.Elapsed;
                    }
                }
                catch (Exception ex)
                {
                    _running = false;
                    string message = ex.Message;
                    // Stop rather than spamming the same exception every
                    // tick - matches the console/Raylib build's behavior of
                    // halting and printing on a core exception rather than
                    // trying to recover. StatusText is a UI element, so the
                    // update has to go through the UI thread.
                    Dispatcher.UIThread.Post(() => StatusText.Text = $"[CPU HALT] {message}");
                    break;
                }

                TimeSpan remaining = nextTick - clock.Elapsed;
                if (remaining > TimeSpan.Zero)
                {
                    SleepUntil(nextTick, clock);
                }
                else
                {
                    // Fell behind - resync to "now" instead of trying to
                    // burst-catch-up, which would just run a pile of frames
                    // back-to-back with no pacing at all.
                    nextTick = clock.Elapsed;
                }
            }
        }

        // A hybrid Sleep-then-spin (sleep for most of the remaining time,
        // busy-spin only the last ~2ms) turned out not to be enough here -
        // measured CPU usage while stuck at ~56fps was only ~5%, meaning
        // the process is nowhere near compute-bound; the shortfall is
        // entirely from oversleeping, not slow work. That means
        // Thread.Sleep's wakeup latency in this environment is bigger than
        // the 2ms margin the hybrid approach budgeted for it - rather than
        // guess at a bigger margin, spin-wait the ENTIRE remaining time
        // instead of calling Thread.Sleep at all. With this much headroom
        // (a handful of percent of one core, going by that measurement),
        // pegging a single core for the ~14ms/frame this spins is a
        // perfectly reasonable trade for hitting the 60fps target
        // precisely - the same tradeoff real-time audio/emulation loops
        // routinely make, at the cost of that one core's power draw.
        private static void SleepUntil(TimeSpan target, Stopwatch clock)
        {
            while (clock.Elapsed < target)
            {
                Thread.SpinWait(100);
            }
        }

        // Called from the emulation thread. Publishes the newest frame and
        // schedules a UI-thread Present only if one isn't already pending -
        // see _pendingFrame's own field comment for why.
        private void SubmitFrame(byte[] pixels, int width, int height)
        {
            Interlocked.Exchange(ref _pendingFrame, new FrameData { Pixels = pixels, Width = width, Height = height });

            if (Interlocked.CompareExchange(ref _presentScheduled, 1, 0) == 0)
            {
                Dispatcher.UIThread.Post(PresentPendingFrame);
            }
        }

        // Runs on the UI thread. Always presents whatever the newest frame
        // is at the moment it actually runs, which may not be the same
        // frame that triggered this dispatch if the emulation thread has
        // since produced newer ones - that's the intended drop-stale-frames
        // behavior, not a bug. GameFrameControl.UpdateFrame takes the raw
        // buffer + dimensions directly (see EmuSen.Serenity), so pseudo-
        // hi-res width changes (SETINI bit 3) just fall out for free -
        // unlike the old WriteableBitmap this replaced, there's no fixed-
        // size backing surface to resize.
        private void PresentPendingFrame()
        {
            try
            {
                FrameData? frame = Interlocked.Exchange(ref _pendingFrame, null);
                if (frame is null) return;
                GameFrame.UpdateFrame(frame.Pixels, frame.Width, frame.Height);
            }
            finally
            {
                // Reset only after the copy/invalidate above finishes, so
                // while a Present is actually running, _emuThread keeps
                // overwriting _pendingFrame without scheduling another one -
                // the coalescing this whole mechanism exists for.
                Interlocked.Exchange(ref _presentScheduled, 0);
            }
        }

        private void OnExitClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }

        // Reuses the console/Raylib build's own CategorizedLogWriter
        // (Common/CategorizedLogWriter.cs) rather than building a second
        // logging mechanism - it works by redirecting Console.Out, and
        // the emulation core already writes every diagnostic message
        // (Cartridge load info, DebugSettings-gated traces, etc.) via
        // plain Console.WriteLine, so this frontend gets the exact same
        // categorized cpu/ppu/apu/memory/debug/general log files the
        // console build does, for free.
        //
        // Unconditional, same as the console/Raylib build
        // (Hotaru/Program.cs) - several DebugSettings.*Logging
        // flags default to true (Dma/CameraRam/RenderRead/AllScrollWrite/
        // MathUnit/BgModeChange/MosaicWrite), so the core writes a steady
        // stream of Console.WriteLine calls regardless of whether anyone
        // asked for logging. Leaving Console.Out un-redirected in that
        // case doesn't turn logging off - it just sends the same volume
        // of lines to the raw, synchronous console writer one syscall at
        // a time instead of CategorizedLogWriter's batched background
        // thread, which is exactly the gap that made this frontend so
        // much slower than the console build. Falling back to an
        // AppSettings.LogDirectory default under %AppData% (rather than
        // skipping the redirect) keeps the fast path always on; the user
        // only needs to set LogDirectory in Preferences if they want the
        // files somewhere specific.
        //
        // Called once per LoadRom() (not once at app startup) since,
        // unlike the console build (one ROM per process), this frontend
        // can load several ROMs across one running session - each gets
        // its own timestamped directory, mirroring the console build's
        // Logs/<CoreName>/console_<timestamp>/ convention but under the
        // user-configured (or default) root and with a "gui_" prefix
        // instead.
        private void StartLogging(string coreName)
        {
            StopLogging(); // close the previous session's files first - see CategorizedLogWriter.Dispose's own comment

            string logRoot = string.IsNullOrWhiteSpace(_appSettings.LogDirectory)
                ? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "EmuSen", "Logs")
                : _appSettings.LogDirectory;

            try
            {
                string logDir = Path.Combine(logRoot, coreName, $"gui_{DateTime.Now:yyyyMMdd_HHmmss}");
                Directory.CreateDirectory(logDir);
                _activeLogWriter = new CategorizedLogWriter(_originalConsoleOut, logDir);
                Console.SetOut(_activeLogWriter);
            }
            catch (Exception ex)
            {
                // Best-effort - an unwritable log directory shouldn't block
                // loading the ROM itself, just leave logging off for this
                // session and say why in the status bar.
                StatusText.Text = $"Logging disabled: {ex.Message}";
            }
        }

        private void StopLogging()
        {
            if (_activeLogWriter is null) return;

            // Must happen before Console.SetOut/Dispose below - see
            // EmulatorSession.FlushVerboseLogs()'s own comment.
            _session?.FlushVerboseLogs();

            Console.SetOut(_originalConsoleOut);
            _activeLogWriter.Dispose();
            _activeLogWriter = null;
        }
    }
}
