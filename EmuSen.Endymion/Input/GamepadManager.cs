using System;
using System.Diagnostics;
using SDL3;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
{
    // Polls the first connected SDL3 gamepad into PadButton state - see EmuSen_Input.md §4.
    public class GamepadManager : IDisposable
    {
        // Swapped when a ROM for a different console loads - see EmuSen_Input.md §5.1.
        public GamepadBindingMap Bindings { get; set; }

        private IntPtr _gamepad;
        private bool _available;
        private bool _sdlInitialized, _started;

        // Rate-limits the hot-plug rescan in Poll() - see EmuSen_Settings_Reference.md §4.4.
        private static readonly TimeSpan RescanInterval = TimeSpan.FromSeconds(1);
        private readonly Stopwatch _rescanClock = Stopwatch.StartNew();
        private TimeSpan _lastRescan = -RescanInterval;

        // start: false leaves SDL untouched until Start, which a window calls once it is shown - see EmuSen_Settings_Reference.md §4.42.
        public GamepadManager(GamepadBindingMap bindings, bool start = true)
        {
            Bindings = bindings;
            if (start) Start();
        }

        public bool Started => _started;

        // Read in place of the device when set, so a test drives the same path a real pad does - see EmuSen_Settings_Reference.md §4.45.
        public SimulatedPad? Simulated { get; set; }

        private bool Button(SDL.GamepadButton button) =>
            Simulated is { } pad ? pad.IsHeld(button) : SDL.GetGamepadButton(_gamepad, button);

        private short RawAxisValue(SDL.GamepadAxis axis) =>
            Simulated is { } pad ? pad.Axis(axis) : SDL.GetGamepadAxis(_gamepad, axis);

        // Idempotent, and on the thread that polls, as SDL asks.
        public void Start()
        {
            if (_started) return;
            _started = true;

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

        // Whether a rescan is due; the subtraction is where a MinValue start overflowed - see EmuSen_Settings_Reference.md §4.4.
        internal static bool RescanDue(TimeSpan now, TimeSpan last) => now - last >= RescanInterval;

        // Call once per frame tick; rescans for a hot-plugged pad at most
        // once per RescanInterval - see EmuSen_Settings_Reference.md §4.4.
        public void Poll()
        {
            if (Simulated is not null || !_sdlInitialized) return;

            // A pad pulled out is let go, so the rescan below can open whichever pad is plugged in next.
            if (_available && !SDL.GamepadConnected(_gamepad))
            {
                SDL.CloseGamepad(_gamepad);
                _gamepad = IntPtr.Zero;
                _available = false;
            }

            if (!_available)
            {
                TimeSpan now = _rescanClock.Elapsed;
                if (RescanDue(now, _lastRescan))
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

        public bool IsConnected => Simulated is not null || (_available && _gamepad != IntPtr.Zero);

        // Null when nothing is connected - see EmuSen_Settings_Reference.md §4.4.
        public string? ControllerName
        {
            get
            {
                if (!IsConnected) return null;
                if (Simulated is { } pad) return pad.Name;
                string? name = SDL.GetGamepadName(_gamepad);
                return string.IsNullOrEmpty(name) ? "Unknown controller" : name;
            }
        }

        // SDL's own reading of what the pad is (Xbox, PlayStation, Nintendo, or not known), from its vendor and product - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType ControllerType =>
            !IsConnected ? SDL.GamepadType.Unknown : Simulated is { } pad ? pad.Type : SDL.GetGamepadType(_gamepad);

        // The connected pad's own printed label for a button, falling back to
        // its position - see EmuSen_Settings_Reference.md §4.6.
        public string? ButtonLabel(SDL.GamepadButton button)
        {
            if (!IsConnected || Simulated is not null) return null;
            SDL.GamepadButtonLabel label = SDL.GetGamepadButtonLabel(_gamepad, button);
            return label == SDL.GamepadButtonLabel.Unknown ? null : label.ToString();
        }

        // Set when the loaded console reads the left stick as a stick, so it stops standing in for the d-pad - see EmuSen_Input.md §7.3.
        public bool LeftStickIsAnalog { get; set; }

        // Below this share of its travel an axis reads zero, so a pad at rest does not drift - see EmuSen_Input.md §7.3.
        public double AnalogDeadzone { get; set; } = 0.1;

        public bool IsPressed(PadButton button)
        {
            if (!IsConnected) return false;

            if (AnalogStickAsDpad && !LeftStickIsAnalog && StickDirectionPressed(button)) return true;

            // A trigger is an axis to SDL, so L2 and R2 are its press past half its travel.
            if (button is PadButton.L2 or PadButton.R2 && Axis(button == PadButton.L2 ? PadAxis.LeftTrigger : PadAxis.RightTrigger) >= 0.5) return true;

            if (!Bindings.ButtonToPad.TryGetValue(button, out SDL.GamepadButton sdlButton)) return false;

            return Button(sdlButton);
        }

        // Sticks -1 to 1 with right and down positive, as SDL and the RetroPad have them, and triggers 0 to 1 - see EmuSen_Input.md §7.
        public double Axis(PadAxis axis)
        {
            if (!IsConnected) return 0;

            SDL.GamepadAxis source = axis switch
            {
                PadAxis.LeftX => SDL.GamepadAxis.LeftX,
                PadAxis.LeftY => SDL.GamepadAxis.LeftY,
                PadAxis.RightX => SDL.GamepadAxis.RightX,
                PadAxis.RightY => SDL.GamepadAxis.RightY,
                PadAxis.LeftTrigger => SDL.GamepadAxis.LeftTrigger,
                _ => SDL.GamepadAxis.RightTrigger,
            };

            double value = Math.Clamp(RawAxisValue(source) / (double)short.MaxValue, -1.0, 1.0);
            return Math.Abs(value) < AnalogDeadzone ? 0 : value;
        }

        // Axis range is -32768..32767; the deadzone is a fraction of it.
        private bool StickDirectionPressed(PadButton button)
        {
            short threshold = (short)(Math.Clamp(StickDeadzone, 0.05, 0.95) * short.MaxValue);

            return button switch
            {
                PadButton.Left => RawAxisValue(SDL.GamepadAxis.LeftX) < -threshold,
                PadButton.Right => RawAxisValue(SDL.GamepadAxis.LeftX) > threshold,
                PadButton.Up => RawAxisValue(SDL.GamepadAxis.LeftY) < -threshold,
                PadButton.Down => RawAxisValue(SDL.GamepadAxis.LeftY) > threshold,
                _ => false,
            };
        }

        // The pad's own buttons and axes, whatever a console's bindings say, for steering the interface - see EmuSen_Settings_Reference.md §4.29.
        public bool IsRawPressed(SDL.GamepadButton button) => IsConnected && Button(button);

        public double RawAxis(SDL.GamepadAxis axis) =>
            IsConnected ? Math.Clamp(RawAxisValue(axis) / (double)short.MaxValue, -1.0, 1.0) : 0;

        // First currently-held pad button, for InputSettingsWindow's rebind
        // capture - see EmuSen_Settings_Reference.md §4.6.
        public SDL.GamepadButton? GetAnyPressedButton()
        {
            if (!IsConnected) return null;

            if (Simulated is null) SDL.UpdateGamepads();
            foreach (SDL.GamepadButton b in Enum.GetValues<SDL.GamepadButton>())
            {
                if (b == SDL.GamepadButton.Invalid || b == SDL.GamepadButton.Count) continue;
                if (Button(b)) return b;
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
