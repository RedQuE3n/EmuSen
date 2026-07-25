using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.TestingStudio.Input;
using EmuSen.TestingStudio.Settings;

namespace EmuSen.TestingStudio.Views
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
        private WriteableBitmap? _bitmap;

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
        }
        private FrameData? _pendingFrame;
        private int _presentScheduled;

        private string? _currentRomPath;
        private readonly ControllerKeyMap _keyBindings = ControllerKeyMap.Load();
        private readonly GamepadBindingMap _gamepadBindings = GamepadBindingMap.Load();
        private readonly AppSettings _appSettings = AppSettings.Load();
        private readonly GamepadManager _gamepad;

        // Captured once at startup, before anything ever redirects
        // Console.Out - restoring this (rather than whatever
        // _activeLogWriter happened to be at the time) is what lets logging
        // be turned off again (or repointed) without leaving Console.Out
        // pointed at a disposed writer.
        private readonly TextWriter _originalConsoleOut = Console.Out;
        private CategorizedLogWriter? _activeLogWriter;

        // Keyboard and gamepad are tracked separately and combined with OR
        // logic - matches the console/Raylib build's own InputBindings.cs
        // convention ("either device works at any time, no need to pick
        // one"). Without this, a gamepad poll finding a button NOT pressed
        // would incorrectly release a button still being held on the
        // keyboard, and vice versa.
        private readonly bool[] _keyboardHeld = new bool[Enum.GetValues<SnesButton>().Length];
        private readonly bool[] _gamepadHeld = new bool[Enum.GetValues<SnesButton>().Length];

        public MainWindow()
        {
            InitializeComponent();
            _gamepad = new GamepadManager(_gamepadBindings);
            Closing += (_, _) =>
            {
                _timer?.Stop();
                StopEmulationThread();
                _session?.SaveSram();
                _gamepad.Dispose();
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
            new InputSettingsWindow(_keyBindings, _gamepadBindings, _gamepad).Show(this);
        }

        private void OnPreferencesClick(object? sender, RoutedEventArgs e)
        {
            new PreferencesWindow(_appSettings).Show(this);
        }

        private void OnDebugLoggingClick(object? sender, RoutedEventArgs e)
        {
            new DebugSettingsWindow().Show(this);
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

                _bitmap = new WriteableBitmap(
                    new PixelSize(_session.ScreenWidth, EmulatorSession.ScreenHeight),
                    new Vector(96, 96),
                    Avalonia.Platform.PixelFormat.Rgba8888,
                    AlphaFormat.Opaque);

                GameView.Source = _bitmap;
                GameView.IsVisible = true;
                NoRomText.IsVisible = false;

                StatusText.Text = $"Running: {displayName}";
                _currentRomPath = path;

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
            _emuThread.Join();
            _emuThread = null;
        }

        // Runs entirely off the UI thread: RunFrame() (the actual CPU/PPU/
        // APU work) and the frame-buffer readout no longer compete with
        // Avalonia's input/paint pump the way they did inside the old
        // DispatcherTimer tick. Only the final bitmap copy + present
        // (PresentPendingFrame, dispatched below) needs the UI thread,
        // since WriteableBitmap is UI-thread-affine.
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

            while (_running)
            {
                nextTick += FrameInterval;

                EmulatorSession? session = _session;
                if (session is null) break;

                try
                {
                    session.RunFrame();
                    byte[] frame = session.GetFrameBufferRgba();
                    SubmitFrame(frame, session.ScreenWidth);

                    framesInWindow++;
                    TimeSpan windowElapsed = clock.Elapsed - fpsWindowStart;
                    if (windowElapsed >= TimeSpan.FromSeconds(1))
                    {
                        double fps = framesInWindow / windowElapsed.TotalSeconds;
                        Dispatcher.UIThread.Post(() => FpsText.Text = $"{fps:F1} fps");
                        framesInWindow = 0;
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
                    Thread.Sleep(remaining);
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

        // Called from the emulation thread. Publishes the newest frame and
        // schedules a UI-thread Present only if one isn't already pending -
        // see _pendingFrame's own field comment for why.
        private void SubmitFrame(byte[] pixels, int width)
        {
            Interlocked.Exchange(ref _pendingFrame, new FrameData { Pixels = pixels, Width = width });

            if (Interlocked.CompareExchange(ref _presentScheduled, 1, 0) == 0)
            {
                Dispatcher.UIThread.Post(PresentPendingFrame);
            }
        }

        // Runs on the UI thread. Always presents whatever the newest frame
        // is at the moment it actually runs, which may not be the same
        // frame that triggered this dispatch if the emulation thread has
        // since produced newer ones - that's the intended drop-stale-frames
        // behavior, not a bug.
        private void PresentPendingFrame()
        {
            try
            {
                FrameData? frame = Interlocked.Exchange(ref _pendingFrame, null);
                if (frame is null || _bitmap is null) return;

                // The frame can now be a different width than the bitmap
                // was created with - pseudo-hi-res (SETINI bit 3) makes a
                // frame 512 pixels wide instead of 256, and a game can
                // toggle it between frames. Recreate the bitmap whenever
                // the byte length doesn't match what's currently allocated,
                // rather than trusting the old fixed 256x224 assumption and
                // Marshal.Copy-ing past the end of a too-small buffer -
                // that used to be a real crash/corruption risk here, not
                // just a cosmetic issue, since Marshal.Copy has no bounds
                // checking of its own.
                int expectedBytes = _bitmap.PixelSize.Width * _bitmap.PixelSize.Height * 4;
                if (frame.Pixels.Length != expectedBytes)
                {
                    _bitmap = new WriteableBitmap(
                        new PixelSize(frame.Width, EmulatorSession.ScreenHeight),
                        new Vector(96, 96),
                        Avalonia.Platform.PixelFormat.Rgba8888,
                        AlphaFormat.Opaque);
                    GameView.Source = _bitmap;
                }

                using (ILockedFramebuffer fb = _bitmap.Lock())
                {
                    Marshal.Copy(frame.Pixels, 0, fb.Address, frame.Pixels.Length);
                }

                GameView.InvalidateVisual();
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
        // (RaylibFrontend/Program.cs) - several DebugSettings.*Logging
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
