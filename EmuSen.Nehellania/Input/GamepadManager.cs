using System;
using System.Diagnostics;
using SDL3;
using EmuSen.Galaxia.Input;

namespace EmuSen.Nehellania.Input
{
    // Polls the first connected SDL3 gamepad into PadButton state - see EmuSen_Input.md §4.
    public class GamepadManager : IDisposable
    {
        // Swapped when a ROM for a different console loads - see EmuSen_Input.md §5.1.
        public GamepadBindingMap Bindings { get; set; }

        private IntPtr _gamepad;
        private bool _available;
        private readonly bool _sdlInitialized;

        // Rate-limits the hot-plug rescan in Poll() - see EmuSen_Settings_Reference.md §4.4.
        private static readonly TimeSpan RescanInterval = TimeSpan.FromSeconds(1);
        private readonly Stopwatch _rescanClock = Stopwatch.StartNew();
        private TimeSpan _lastRescan = TimeSpan.MinValue;

        public GamepadManager(GamepadBindingMap bindings)
        {
            Bindings = bindings;

            // Gamepad subsystem only, and InitSubSystem rather than Init -
            // see EmuSen_Settings_Reference.md §4.10.
            _sdlInitialized = SDL.InitSubSystem(SDL.InitFlags.Gamepad);
            if (!_sdlInitialized) return;

            TryOpenFirstController();
        }

        private void TryOpenFirstController()
        {
            // SDL3 enumerates gamepads directly, no joystick-index filtering.
            uint[]? pads = SDL.GetGamepads(out int count);
            for (int i = 0; pads is not null && i < count; i++)
            {
                _gamepad = SDL.OpenGamepad(pads[i]);
                if (_gamepad != IntPtr.Zero)
                {
                    _available = true;
                    return;
                }
            }
            _available = false;
        }

        // Call once per frame tick; rescans for a hot-plugged pad at most
        // once per RescanInterval - see EmuSen_Settings_Reference.md §4.4.
        public void Poll()
        {
            if (!_sdlInitialized) return;

            if (!_available)
            {
                TimeSpan now = _rescanClock.Elapsed;
                if (now - _lastRescan >= RescanInterval)
                {
                    _lastRescan = now;
                    TryOpenFirstController();
                }
            }
            SDL.UpdateGamepads();
        }

        // Stick-as-d-pad and its threshold - see EmuSen_Settings_Reference.md §4.4.
        public bool AnalogStickAsDpad { get; set; } = true;
        public double StickDeadzone { get; set; } = 0.5;

        public bool IsConnected => _available && _gamepad != IntPtr.Zero;

        // Null when nothing is connected - see EmuSen_Settings_Reference.md §4.4.
        public string? ControllerName
        {
            get
            {
                if (!IsConnected) return null;
                string? name = SDL.GetGamepadName(_gamepad);
                return string.IsNullOrEmpty(name) ? "Unknown controller" : name;
            }
        }

        // The connected pad's own printed label for a button, falling back to
        // its position - see EmuSen_Settings_Reference.md §4.6.
        public string? ButtonLabel(SDL.GamepadButton button)
        {
            if (!IsConnected) return null;
            SDL.GamepadButtonLabel label = SDL.GetGamepadButtonLabel(_gamepad, button);
            return label == SDL.GamepadButtonLabel.Unknown ? null : label.ToString();
        }

        public bool IsPressed(PadButton button)
        {
            if (!IsConnected) return false;

            if (AnalogStickAsDpad && StickDirectionPressed(button)) return true;

            if (!Bindings.ButtonToPad.TryGetValue(button, out SDL.GamepadButton sdlButton)) return false;

            return SDL.GetGamepadButton(_gamepad, sdlButton);
        }

        // Axis range is -32768..32767; the deadzone is a fraction of it.
        private bool StickDirectionPressed(PadButton button)
        {
            short threshold = (short)(Math.Clamp(StickDeadzone, 0.05, 0.95) * short.MaxValue);

            return button switch
            {
                PadButton.Left => SDL.GetGamepadAxis(_gamepad, SDL.GamepadAxis.LeftX) < -threshold,
                PadButton.Right => SDL.GetGamepadAxis(_gamepad, SDL.GamepadAxis.LeftX) > threshold,
                PadButton.Up => SDL.GetGamepadAxis(_gamepad, SDL.GamepadAxis.LeftY) < -threshold,
                PadButton.Down => SDL.GetGamepadAxis(_gamepad, SDL.GamepadAxis.LeftY) > threshold,
                _ => false,
            };
        }

        // First currently-held pad button, for InputSettingsWindow's rebind
        // capture - see EmuSen_Settings_Reference.md §4.6.
        public SDL.GamepadButton? GetAnyPressedButton()
        {
            if (!IsConnected) return null;

            SDL.UpdateGamepads();
            foreach (SDL.GamepadButton b in Enum.GetValues<SDL.GamepadButton>())
            {
                if (b == SDL.GamepadButton.Invalid || b == SDL.GamepadButton.Count) continue;
                if (SDL.GetGamepadButton(_gamepad, b)) return b;
            }
            return null;
        }

        public void Dispose()
        {
            if (_gamepad != IntPtr.Zero)
            {
                SDL.CloseGamepad(_gamepad);
                _gamepad = IntPtr.Zero;
            }
            // QuitSubSystem, not Quit - see EmuSen_Settings_Reference.md §4.10.
            if (_sdlInitialized) SDL.QuitSubSystem(SDL.InitFlags.Gamepad);
        }
    }
}
