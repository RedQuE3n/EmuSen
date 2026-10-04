using System;
using System.Collections.Generic;
using System.Diagnostics;
using SDL3;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
{
    // Polls every connected SDL3 gamepad and seats each as a player - see EmuSen_Input.md §4 and §8, and EmuSen_Settings_Reference.md §4.61.
    public class GamepadManager : IDisposable
    {
        // Swapped when a ROM for a different console loads - see EmuSen_Input.md §5.1.
        public GamepadBindingMap Bindings { get; set; }

        // Players 2 on; null, or a null answer, gives them player 1's map - see EmuSen_Input.md §8.4.
        public Func<int, GamepadBindingMap?>? PlayerBindings { get; set; }

        public GamepadBindingMap BindingsFor(int player) => player > 1 && PlayerBindings?.Invoke(player) is { } map ? map : Bindings;

        // Which pad is which player - see EmuSen_Input.md §8.
        public PlayerSlots Players { get; } = new();

        private IPadDevices _devices;
        private readonly List<ConnectedPad> _pads = new();
        private bool _sdlInitialized, _started;

        // Rate-limits the hot-plug rescan in Poll() - see EmuSen_Settings_Reference.md §4.4.
        private static readonly TimeSpan RescanInterval = TimeSpan.FromSeconds(1);
        private readonly Stopwatch _rescanClock = Stopwatch.StartNew();
        private TimeSpan _lastRescan = -RescanInterval;

        // start: false leaves SDL untouched until Start, which a window calls once it is shown - see EmuSen_Settings_Reference.md §4.42.
        public GamepadManager(GamepadBindingMap bindings, bool start = true, IPadDevices? devices = null)
        {
            Bindings = bindings;
            _devices = devices ?? new SdlPadDevices();
            if (start) Start();
        }

        public bool Started => _started;

        // Raised from Poll for each pad plugged in or pulled out after Start; the pads present at Start are not announced.
        public event Action<PadConnection>? PadChanged;

        // Raised at the end of each Poll, so a window that shows the pad reads it on the one poll there is - see EmuSen_Settings_Reference.md §4.81.
        public event Action? Polled;

        // Every open pad, in the order they were opened; the first is the first controller, which need not be player 1's.
        public IReadOnlyList<ConnectedPad> Pads => _pads;

        public ConnectedPad? Primary => _pads.Count > 0 ? _pads[0] : null;

        // ES-DE's "Only accept input from first controller": the interface reads the first pad alone - see EmuSen_Settings_Reference.md §4.61.
        public bool FirstControllerOnly { get; set; }

        // How many of Pads, from the first, the interface reads.
        public int FrontendPadCount => FirstControllerOnly ? Math.Min(1, _pads.Count) : _pads.Count;

        // Read in place of the device when set, so a test drives the same path a real pad does - see EmuSen_Settings_Reference.md §4.45.
        public SimulatedPad? Simulated
        {
            get => (_devices as SimulatedPads)?.First;
            set => UseDevices(value is null ? new SdlPadDevices() : SimulatedPads.With(value));
        }

        public IPadDevices Devices => _devices;

        // Lets go of every pad on the old devices and starts on the new ones.
        public void UseDevices(IPadDevices devices)
        {
            CloseAll();
            Players.Clear();
            _started = false;
            _devices = devices;
            Start();
        }

        // Idempotent, and on the thread that polls, as SDL asks.
        public void Start()
        {
            if (_started) return;
            _started = true;

            _sdlInitialized = _devices.Init();
            if (!_sdlInitialized) return;

            OpenAttached(announce: false);
        }

        private void OpenAttached(bool announce)
        {
            foreach (uint id in _devices.Attached())
            {
                if (_pads.Exists(p => p.Id == id)) continue;
                IntPtr handle = _devices.Open(id);
                if (handle == IntPtr.Zero) continue;
                var pad = new ConnectedPad(_devices, id, handle);
                _pads.Add(pad);
                int player = Players.Seat(pad);
                LightPlayers();
                if (announce) PadChanged?.Invoke(new PadConnection(pad, Connected: true) { Player = player });
            }
        }

        // Whether a rescan is due; the subtraction is where a MinValue start overflowed - see EmuSen_Settings_Reference.md §4.4.
        internal static bool RescanDue(TimeSpan now, TimeSpan last) => now - last >= RescanInterval;

        // Call once per frame tick: lets go of pads pulled out, and opens new ones on SDL's events or the one-a-second rescan - see EmuSen_Settings_Reference.md §4.4 and §4.61.
        public void Poll()
        {
            if (!_sdlInitialized) return;

            for (int i = 0; i < _pads.Count; i++)
            {
                ConnectedPad pad = _pads[i];
                if (_devices.IsAttached(pad.Handle)) continue;
                _pads.RemoveAt(i--);
                pad.Close();
                PadChanged?.Invoke(new PadConnection(pad, Connected: false) { Player = Players.PlayerOf(pad) });
            }

            TimeSpan now = _rescanClock.Elapsed;
            if (_devices.DevicesChanged() | RescanDue(now, _lastRescan))
            {
                _lastRescan = now;
                OpenAttached(announce: true);
            }
            _devices.Update();
            Polled?.Invoke();
        }

        // The player chose a seat for the pad, 0 for none; the pads that light a number show the new ones.
        public void Assign(ConnectedPad pad, int player)
        {
            Players.Move(pad, player);
            LightPlayers();
        }

        private void LightPlayers()
        {
            foreach (ConnectedPad pad in _pads)
                if (pad.IsOpen) _devices.SetPlayerIndex(pad.Handle, Players.PlayerOf(pad) - 1);
        }

        // The pad the game hears as the player; with the first controller alone, player 1's only - see EmuSen_Input.md §8.3.
        public ConnectedPad? PlayerPad(int player) => FirstControllerOnly && player > 1 ? null : Players.PadFor(player);

        // Stick-as-d-pad and its threshold - see EmuSen_Settings_Reference.md §4.4.
        public bool AnalogStickAsDpad { get; set; } = true;
        public double StickDeadzone { get; set; } = 0.5;

        public bool IsConnected => _pads.Count > 0;

        // Player 1's pad's name; null when nothing is connected - see EmuSen_Settings_Reference.md §4.4.
        public string? ControllerName => Primary?.Name;

        // SDL's own reading of what player 1's pad is (Xbox, PlayStation, Nintendo, or not known), from its vendor and product - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType ControllerType => Primary?.Type ?? SDL.GamepadType.Unknown;

        // Player 1's pad's own printed label for a button, falling back to its position - see EmuSen_Settings_Reference.md §4.6.
        public string? ButtonLabel(SDL.GamepadButton button) => Primary?.ButtonLabel(button);

        // Set when the loaded console reads the left stick as a stick, so it stops standing in for the d-pad - see EmuSen_Input.md §7.3.
        public bool LeftStickIsAnalog { get; set; }

        // Below this share of its travel an axis reads zero, so a pad at rest does not drift - see EmuSen_Input.md §7.3.
        public double AnalogDeadzone { get; set; } = 0.1;

        public bool IsPressed(PadButton button) => IsPressed(button, 1);

        // Through the player's own bindings, on the player's own pad.
        public bool IsPressed(PadButton button, int player)
        {
            if (PlayerPad(player) is not { } pad) return false;

            if (AnalogStickAsDpad && !LeftStickIsAnalog && StickDirectionPressed(pad, button)) return true;

            // A trigger is an axis to SDL, so L2 and R2 are its press past half its travel.
            if (button is PadButton.L2 or PadButton.R2 && Axis(button == PadButton.L2 ? PadAxis.LeftTrigger : PadAxis.RightTrigger, player) >= 0.5) return true;

            if (!BindingsFor(player).ButtonToPad.TryGetValue(button, out SDL.GamepadButton sdlButton)) return false;

            return pad.IsRawPressed(sdlButton);
        }

        // Sticks -1 to 1 with right and down positive, as SDL and the RetroPad have them, and triggers 0 to 1 - see EmuSen_Input.md §7.
        public double Axis(PadAxis axis) => Axis(axis, 1);

        public double Axis(PadAxis axis, int player)
        {
            if (PlayerPad(player) is not { } pad) return 0;

            SDL.GamepadAxis source = axis switch
            {
                PadAxis.LeftX => SDL.GamepadAxis.LeftX,
                PadAxis.LeftY => SDL.GamepadAxis.LeftY,
                PadAxis.RightX => SDL.GamepadAxis.RightX,
                PadAxis.RightY => SDL.GamepadAxis.RightY,
                PadAxis.LeftTrigger => SDL.GamepadAxis.LeftTrigger,
                _ => SDL.GamepadAxis.RightTrigger,
            };

            double value = pad.RawAxis(source);
            return Math.Abs(value) < AnalogDeadzone ? 0 : value;
        }

        // Axis range is -32768..32767; the deadzone is a fraction of it.
        private bool StickDirectionPressed(ConnectedPad pad, PadButton button)
        {
            short threshold = (short)(Math.Clamp(StickDeadzone, 0.05, 0.95) * short.MaxValue);

            return button switch
            {
                PadButton.Left => pad.AxisValue(SDL.GamepadAxis.LeftX) < -threshold,
                PadButton.Right => pad.AxisValue(SDL.GamepadAxis.LeftX) > threshold,
                PadButton.Up => pad.AxisValue(SDL.GamepadAxis.LeftY) < -threshold,
                PadButton.Down => pad.AxisValue(SDL.GamepadAxis.LeftY) > threshold,
                _ => false,
            };
        }

        // Player 1's pad's own buttons and axes, whatever a console's bindings say - see EmuSen_Settings_Reference.md §4.29.
        public bool IsRawPressed(SDL.GamepadButton button) => Primary?.IsRawPressed(button) ?? false;

        public double RawAxis(SDL.GamepadAxis axis) => Primary?.RawAxis(axis) ?? 0;

        // First button held on any pad the interface reads, for InputSettingsWindow's rebind capture - see EmuSen_Settings_Reference.md §4.6 and §4.61.
        public SDL.GamepadButton? GetAnyPressedButton()
        {
            if (!IsConnected) return null;

            _devices.Update();
            for (int i = 0; i < FrontendPadCount; i++)
                foreach (SDL.GamepadButton b in Enum.GetValues<SDL.GamepadButton>())
                {
                    if (b == SDL.GamepadButton.Invalid || b == SDL.GamepadButton.Count) continue;
                    if (_pads[i].IsRawPressed(b)) return b;
                }
            return null;
        }

        private void CloseAll()
        {
            foreach (ConnectedPad pad in _pads) pad.Close();
            _pads.Clear();
            // QuitSubSystem, not Quit - see EmuSen_Settings_Reference.md §4.10.
            if (_sdlInitialized) _devices.Quit();
            _sdlInitialized = false;
        }

        public void Dispose() => CloseAll();
    }
}
