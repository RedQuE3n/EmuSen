using System;
using System.Linq;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Controllers;
using EmuSen.Frontend.Input;

namespace EmuSen.Frontend.Views
{
    public partial class MainWindow : Window
    {
        // ~60fps. Not synced to the core's actual scanline timing yet - this
        // is a fixed-interval UI timer, which is fine for a first pass but
        // will drift from real SNES frame timing over long sessions. Worth
        // revisiting (e.g. accumulator-based stepping) once this is otherwise
        // working.
        private static readonly TimeSpan FrameInterval = TimeSpan.FromSeconds(1.0 / 60.0);

        private EmulatorSession? _session;
        private WriteableBitmap? _bitmap;
        private DispatcherTimer? _timer;
        private string? _currentRomPath;
        private readonly ControllerKeyMap _keyBindings = ControllerKeyMap.Load();
        private readonly GamepadBindingMap _gamepadBindings = GamepadBindingMap.Load();
        private readonly GamepadManager _gamepad;

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
                _session?.SaveSram();
                _gamepad.Dispose();
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
            var files = await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
            {
                Title = "Open ROM",
                AllowMultiple = false,
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

        private void OnControllerBindingsClick(object? sender, RoutedEventArgs e)
        {
            new InputSettingsWindow(_keyBindings, _gamepadBindings, _gamepad).Show(this);
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
            _session?.SaveSram(); // flush whatever was previously running before switching

            try
            {
                _session = new EmulatorSession();
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

                _timer = new DispatcherTimer { Interval = FrameInterval };
                _timer.Tick += (_, _) => RunOneFrame();
                _timer.Start();
            }
            catch (Exception ex)
            {
                _session = null;
                StatusText.Text = $"Failed to load {displayName}: {ex.Message}";
            }
        }

        private void RunOneFrame()
        {
            if (_session is null || _bitmap is null) return;

            try
            {
                PollGamepad();

                _session.RunFrame();

                byte[] frame = _session.GetFrameBufferRgba();

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
                if (frame.Length != expectedBytes)
                {
                    _bitmap = new WriteableBitmap(
                        new PixelSize(_session.ScreenWidth, EmulatorSession.ScreenHeight),
                        new Vector(96, 96),
                        Avalonia.Platform.PixelFormat.Rgba8888,
                        AlphaFormat.Opaque);
                    GameView.Source = _bitmap;
                }

                using (ILockedFramebuffer fb = _bitmap.Lock())
                {
                    Marshal.Copy(frame, 0, fb.Address, frame.Length);
                }

                GameView.InvalidateVisual();
            }
            catch (Exception ex)
            {
                // Stop rather than spamming the same exception every tick -
                // matches the console/Raylib build's behavior of halting and
                // printing on a core exception rather than trying to recover.
                _timer?.Stop();
                StatusText.Text = $"[CPU HALT] {ex.Message}";
            }
        }

        private void OnExitClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}
