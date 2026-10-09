using System;
using System.Collections.Generic;
using System.Diagnostics;
using SDL3;
using EmuSen.Endymion.Native;
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
        public PlayerSlots Players { get; }

        // One of the two is set: the C# manager, or the library's pads with the seats and bindings kept here - see EmuSen_RustPlatform.md §14.2.
        private Managed? _managed;
        private Native? _native;

        // start: false leaves SDL untouched until Start, which a window calls once it is shown - see EmuSen_Settings_Reference.md §4.42.
        public GamepadManager(GamepadBindingMap bindings, bool start = true, IPadDevices? devices = null)
            : this(bindings, start, devices, DeviceNative.Active, null, new PlayerSlots()) { }

        // The library's pads or the C#'s, chosen for the instance; devices the library does not know run the C#; only, for a test, the SDL pads the library may see.
        internal GamepadManager(GamepadBindingMap bindings, bool start, IPadDevices? devices, bool native, IReadOnlyCollection<uint>? only = null, PlayerSlots? players = null)
        {
            Bindings = bindings;
            Players = players ?? new PlayerSlots(native);
            devices ??= new SdlPadDevices();
            if (native && Native.Knows(devices)) _native = new Native(this, devices, only);
            else _managed = new Managed(this, devices);
            if (start) Start();
        }

        public bool Started => _managed?.Started ?? _native!.Started;

        // Raised from Poll for each pad plugged in or pulled out after Start; the pads present at Start are not announced.
        public event Action<PadConnection>? PadChanged;

        // Raised at the end of each Poll, so a window that shows the pad reads it on the one poll there is - see EmuSen_Settings_Reference.md §4.81.
        public event Action? Polled;

        // Every open pad, in the order they were opened; the first is the first controller, which need not be player 1's.
        public IReadOnlyList<ConnectedPad> Pads => _managed?.Pads ?? _native!.Pads;

        public ConnectedPad? Primary => Pads.Count > 0 ? Pads[0] : null;

        // ES-DE's "Only accept input from first controller": the interface reads the first pad alone - see EmuSen_Settings_Reference.md §4.61.
        public bool FirstControllerOnly
        {
            get => _managed?.FirstControllerOnly ?? _native!.Flag(DeviceNative.SettingFirstOnly);
            set { if (_managed is not null) _managed.FirstControllerOnly = value; else _native!.Set(DeviceNative.SettingFirstOnly, value ? 1 : 0); }
        }

        // How many of Pads, from the first, the interface reads.
        public int FrontendPadCount => _managed?.FrontendPadCount ?? _native!.FrontendPadCount;

        // Read in place of the device when set, so a test drives the same path a real pad does - see EmuSen_Settings_Reference.md §4.45.
        public SimulatedPad? Simulated
        {
            get => (Devices as SimulatedPads)?.First;
            set => UseDevices(value is null ? new SdlPadDevices() : SimulatedPads.With(value));
        }

        public IPadDevices Devices => _managed?.Devices ?? _native!.Devices;

        // Lets go of every pad on the old devices and starts on the new ones.
        public void UseDevices(IPadDevices devices)
        {
            if (_native is not null && !Native.Knows(devices))
            {
                // Devices the library does not know: this manager is the C#'s from here, with the settings it had.
                var managed = new Managed(this, _native.Devices);
                _native.CopySettingsTo(managed);
                _native.CloseAll();
                _native = null;
                _managed = managed;
            }
            if (_managed is not null) _managed.UseDevices(devices);
            else _native!.UseDevices(devices);
        }

        // Idempotent, and on the thread that polls, as SDL asks.
        public void Start()
        {
            if (_managed is not null) _managed.Start();
            else _native!.Start();
        }

        // Whether a rescan is due; the subtraction is where a MinValue start overflowed - see EmuSen_Settings_Reference.md §4.4.
        internal static bool RescanDue(TimeSpan now, TimeSpan last) => now - last >= Managed.RescanInterval;

        // When the last rescan was, on the manager's own clock; a second before it starts, so the first poll rescans.
        internal TimeSpan LastRescan => _managed?.LastRescan ?? _native!.LastRescan;

        // Call once per frame tick: lets go of pads pulled out, and opens new ones on SDL's events or the one-a-second rescan - see EmuSen_Settings_Reference.md §4.4 and §4.61.
        public void Poll()
        {
            if (_managed is not null) _managed.Poll();
            else _native!.Poll();
        }

        // The player chose a seat for the pad, 0 for none; the pads that light a number show the new ones.
        public void Assign(ConnectedPad pad, int player)
        {
            Players.Move(pad, player);
            if (_managed is not null) _managed.LightPlayers();
            else _native!.LightPlayers();
        }

        // The pad the game hears as the player; with the first controller alone, player 1's only - see EmuSen_Input.md §8.3.
        public ConnectedPad? PlayerPad(int player) => FirstControllerOnly && player > 1 ? null : Players.PadFor(player);

        // Stick-as-d-pad and its threshold - see EmuSen_Settings_Reference.md §4.4.
        public bool AnalogStickAsDpad
        {
            get => _managed?.AnalogStickAsDpad ?? _native!.Flag(DeviceNative.SettingStickAsDpad);
            set { if (_managed is not null) _managed.AnalogStickAsDpad = value; else _native!.Set(DeviceNative.SettingStickAsDpad, value ? 1 : 0); }
        }

        public double StickDeadzone
        {
            get => _managed?.StickDeadzone ?? _native!.Get(DeviceNative.SettingStickDeadzone);
            set { if (_managed is not null) _managed.StickDeadzone = value; else _native!.Set(DeviceNative.SettingStickDeadzone, value); }
        }

        public bool IsConnected => Pads.Count > 0;

        // Player 1's pad's name; null when nothing is connected - see EmuSen_Settings_Reference.md §4.4.
        public string? ControllerName => Primary?.Name;

        // SDL's own reading of what player 1's pad is (Xbox, PlayStation, Nintendo, or not known), from its vendor and product - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType ControllerType => Primary?.Type ?? SDL.GamepadType.Unknown;

        // Player 1's pad's own printed label for a button, falling back to its position - see EmuSen_Settings_Reference.md §4.6.
        public string? ButtonLabel(SDL.GamepadButton button) => Primary?.ButtonLabel(button);

        // Set when the loaded console reads the left stick as a stick, so it stops standing in for the d-pad - see EmuSen_Input.md §7.3.
        public bool LeftStickIsAnalog
        {
            get => _managed?.LeftStickIsAnalog ?? _native!.Flag(DeviceNative.SettingLeftStickAnalog);
            set { if (_managed is not null) _managed.LeftStickIsAnalog = value; else _native!.Set(DeviceNative.SettingLeftStickAnalog, value ? 1 : 0); }
        }

        // Below this share of its travel an axis reads zero, so a pad at rest does not drift - see EmuSen_Input.md §7.3.
        public double AnalogDeadzone
        {
            get => _managed?.AnalogDeadzone ?? _native!.Get(DeviceNative.SettingAnalogDeadzone);
            set { if (_managed is not null) _managed.AnalogDeadzone = value; else _native!.Set(DeviceNative.SettingAnalogDeadzone, value); }
        }

        public bool IsPressed(PadButton button) => IsPressed(button, 1);

        // Through the player's own bindings, on the player's own pad.
        public bool IsPressed(PadButton button, int player) => _managed?.IsPressed(button, player) ?? _native!.IsPressed(button, player);

        // Sticks -1 to 1 with right and down positive, as SDL and the RetroPad have them, and triggers 0 to 1 - see EmuSen_Input.md §7.
        public double Axis(PadAxis axis) => Axis(axis, 1);

        public double Axis(PadAxis axis, int player) => _managed?.Axis(axis, player) ?? _native!.Axis(axis, player);

        // Player 1's pad's own buttons and axes, whatever a console's bindings say - see EmuSen_Settings_Reference.md §4.29.
        public bool IsRawPressed(SDL.GamepadButton button) => Primary?.IsRawPressed(button) ?? false;

        public double RawAxis(SDL.GamepadAxis axis) => Primary?.RawAxis(axis) ?? 0;

        // First button held on any pad the interface reads, for InputSettingsWindow's rebind capture - see EmuSen_Settings_Reference.md §4.6 and §4.61.
        public SDL.GamepadButton? GetAnyPressedButton() => _managed is not null ? _managed.GetAnyPressedButton() : _native!.GetAnyPressedButton();

        public void Dispose()
        {
            if (_managed is not null) _managed.CloseAll();
            else _native!.CloseAll();
        }

        // The library's pads behind this manager: what it opened and closed is seated, lit and announced here, in the order the C# does each.
        private sealed unsafe class Native
        {
            private readonly GamepadManager _owner;
            private readonly NativeHandle _handle;
            private readonly List<ConnectedPad> _pads = new();
            // A test's own list of the SDL pads the library may see, read again at each poll as the test attaches more.
            private readonly IReadOnlyCollection<uint>? _only;

            // SDL's pads, or a simulated set that is the library's own.
            public static bool Knows(IPadDevices devices) => devices is SdlPadDevices || devices is SimulatedPads { IsNative: true };

            public Native(GamepadManager owner, IPadDevices devices, IReadOnlyCollection<uint>? only)
            {
                _owner = owner;
                Devices = devices;
                _only = only;
                uint[]? shown = Shown();
                nint made;
                fixed (uint* o = shown) made = DeviceNative.PadsNew(Kind(devices), Set(devices), shown is null ? null : Only(o), (nuint)(shown?.Length ?? 0));
                if (made == 0) throw new InvalidOperationException(EndymionNative.Words());
                _handle = new NativeHandle(made, DeviceNative.PadsFree);
                GC.KeepAlive(devices);
            }

            public IPadDevices Devices { get; private set; }

            public IReadOnlyList<ConnectedPad> Pads => _pads;

            private static uint Kind(IPadDevices devices) => devices is SimulatedPads ? DeviceNative.DevicesSimulated : DeviceNative.DevicesSdl;

            private static nint Set(IPadDevices devices) => devices is SimulatedPads set ? set.NativeValue : 0;

            private uint[]? Shown() => _only is null ? null : [.. _only];

            // An empty list's pointer is not null: none of SDL's pads, where null is all of them.
            private static uint* Only(uint* pinned) => pinned is null ? (uint*)4 : pinned;

            public bool Started => Number(DeviceNative.PadsStarted) != 0;

            public int FrontendPadCount => (int)Number(DeviceNative.PadsFrontendCount);

            public TimeSpan LastRescan => TimeSpan.FromTicks(Number(DeviceNative.PadsLastRescan));

            public void Start()
            {
                var room = new PadEvent[8];
                long made;
                fixed (PadEvent* e = room) made = DeviceNative.PadsStart(_handle.Value, e, (nuint)room.Length);
                Apply(room, made);
            }

            public void Poll()
            {
                if (Shown() is { } shown)
                    fixed (uint* o = shown) DeviceNative.PadsShowOnly(_handle.Value, Only(o), (nuint)shown.Length);
                var room = new PadEvent[8];
                long made;
                fixed (PadEvent* e = room) made = DeviceNative.PadsPoll(_handle.Value, e, (nuint)room.Length);
                GC.KeepAlive(_handle);
                // Devices that did not start are not polled, and nothing is raised.
                if (made < 0) return;
                Apply(room, made);
                DeviceNative.PadsUpdate(_handle.Value);
                GC.KeepAlive(_handle);
                _owner.Polled?.Invoke();
            }

            public void UseDevices(IPadDevices devices)
            {
                var room = new PadEvent[8];
                uint[]? shown = devices is SdlPadDevices ? Shown() : null;
                long made;
                fixed (uint* o = shown)
                fixed (PadEvent* e = room)
                    made = DeviceNative.PadsUse(_handle.Value, Kind(devices), Set(devices), shown is null ? null : Only(o), (nuint)(shown?.Length ?? 0), e, (nuint)room.Length);
                if (made < 0) throw new InvalidOperationException(EndymionNative.Words());
                GC.KeepAlive(devices);
                Devices = devices;
                _pads.Clear();
                _owner.Players.Clear();
                Apply(room, made);
            }

            // Each pad the library opened is seated, the players lit and the pad announced before the next; each it closed is announced with the seat it keeps.
            private void Apply(PadEvent[] room, long made)
            {
                if (made > room.Length)
                {
                    room = new PadEvent[made];
                    fixed (PadEvent* e = room) DeviceNative.PadsTake(_handle.Value, e, (nuint)room.Length);
                }
                GC.KeepAlive(_handle);
                for (int i = 0; i < made; i++)
                {
                    PadEvent change = room[i];
                    if (change.Kind == 0)
                    {
                        var pad = new ConnectedPad(_handle, change.Key);
                        _pads.Add(pad);
                        int player = _owner.Players.Seat(pad);
                        LightPlayers();
                        if (change.Announce != 0) _owner.PadChanged?.Invoke(new PadConnection(pad, Connected: true) { Player = player });
                    }
                    else if (_pads.Find(p => p.Key == change.Key) is { } gone)
                    {
                        _pads.Remove(gone);
                        _owner.PadChanged?.Invoke(new PadConnection(gone, Connected: false) { Player = _owner.Players.PlayerOf(gone) });
                    }
                }
            }

            public void LightPlayers()
            {
                foreach (ConnectedPad pad in _pads)
                    if (pad.IsOpen) DeviceNative.PadsSetPlayerIndex(_handle.Value, pad.Key, _owner.Players.PlayerOf(pad) - 1);
                GC.KeepAlive(_handle);
            }

            // One crossing when the bindings are a map to look in; a player's own bindings are a delegate, asked only when the stick and the trigger do not answer, as the C# asks it.
            public bool IsPressed(PadButton button, int player)
            {
                if (_owner.PlayerPad(player) is not { } pad) return false;
                bool asks = player > 1 && _owner.PlayerBindings is not null;
                int bound = !asks && _owner.Bindings.ButtonToPad.TryGetValue(button, out SDL.GamepadButton known) ? (int)known : -1;
                bool pressed = DeviceNative.PadsPressed(_handle.Value, pad.Key, (uint)button, bound) != 0;
                GC.KeepAlive(_handle);
                if (pressed || !asks) return pressed;
                if (!_owner.BindingsFor(player).ButtonToPad.TryGetValue(button, out SDL.GamepadButton sdlButton)) return false;
                return pad.IsRawPressed(sdlButton);
            }

            public double Axis(PadAxis axis, int player)
            {
                if (_owner.PlayerPad(player) is not { } pad) return 0;
                double value;
                DeviceNative.PadsAxis(_handle.Value, pad.Key, DeviceNative.AxisPad, (uint)axis, &value);
                GC.KeepAlive(_handle);
                return value;
            }

            public SDL.GamepadButton? GetAnyPressedButton()
            {
                if (_pads.Count == 0) return null;
                long button = Number(DeviceNative.PadsAnyPressed);
                return button < 0 ? null : (SDL.GamepadButton)button;
            }

            public void CloseAll()
            {
                DeviceNative.PadsCloseAll(_handle.Value);
                GC.KeepAlive(_handle);
                _pads.Clear();
            }

            public bool Flag(uint which) => Get(which) != 0;

            public double Get(uint which)
            {
                double value;
                DeviceNative.PadsSetting(_handle.Value, which, &value);
                GC.KeepAlive(_handle);
                return value;
            }

            public void Set(uint which, double value)
            {
                DeviceNative.PadsSetSetting(_handle.Value, which, value);
                GC.KeepAlive(_handle);
            }

            public void CopySettingsTo(Managed managed)
            {
                managed.FirstControllerOnly = Flag(DeviceNative.SettingFirstOnly);
                managed.AnalogStickAsDpad = Flag(DeviceNative.SettingStickAsDpad);
                managed.StickDeadzone = Get(DeviceNative.SettingStickDeadzone);
                managed.LeftStickIsAnalog = Flag(DeviceNative.SettingLeftStickAnalog);
                managed.AnalogDeadzone = Get(DeviceNative.SettingAnalogDeadzone);
            }

            private long Number(uint which)
            {
                long value = DeviceNative.PadsGet(_handle.Value, 0, which, 0);
                GC.KeepAlive(_handle);
                return value;
            }
        }

        // The C# manager: the default, and what the library's pads are held to - see EmuSen_RustPlatform.md §14.
        internal sealed class Managed
        {
            private readonly GamepadManager _owner;
            private IPadDevices _devices;
            private readonly List<ConnectedPad> _pads = new();
            private bool _sdlInitialized, _started;

            // Rate-limits the hot-plug rescan in Poll() - see EmuSen_Settings_Reference.md §4.4.
            internal static readonly TimeSpan RescanInterval = TimeSpan.FromSeconds(1);
            private readonly Stopwatch _rescanClock = Stopwatch.StartNew();
            private TimeSpan _lastRescan = -RescanInterval;

            public Managed(GamepadManager owner, IPadDevices devices)
            {
                _owner = owner;
                _devices = devices;
            }

            public bool Started => _started;

            public TimeSpan LastRescan => _lastRescan;

            public IReadOnlyList<ConnectedPad> Pads => _pads;

            public bool FirstControllerOnly { get; set; }

            public int FrontendPadCount => FirstControllerOnly ? Math.Min(1, _pads.Count) : _pads.Count;

            public IPadDevices Devices => _devices;

            public void UseDevices(IPadDevices devices)
            {
                CloseAll();
                _owner.Players.Clear();
                _started = false;
                _devices = devices;
                Start();
            }

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
                    int player = _owner.Players.Seat(pad);
                    LightPlayers();
                    if (announce) _owner.PadChanged?.Invoke(new PadConnection(pad, Connected: true) { Player = player });
                }
            }

            public void Poll()
            {
                if (!_sdlInitialized) return;

                for (int i = 0; i < _pads.Count; i++)
                {
                    ConnectedPad pad = _pads[i];
                    if (_devices.IsAttached(pad.Handle)) continue;
                    _pads.RemoveAt(i--);
                    pad.Close();
                    _owner.PadChanged?.Invoke(new PadConnection(pad, Connected: false) { Player = _owner.Players.PlayerOf(pad) });
                }

                TimeSpan now = _rescanClock.Elapsed;
                if (_devices.DevicesChanged() | RescanDue(now, _lastRescan))
                {
                    _lastRescan = now;
                    OpenAttached(announce: true);
                }
                _devices.Update();
                _owner.Polled?.Invoke();
            }

            public void LightPlayers()
            {
                foreach (ConnectedPad pad in _pads)
                    if (pad.IsOpen) _devices.SetPlayerIndex(pad.Handle, _owner.Players.PlayerOf(pad) - 1);
            }

            public bool AnalogStickAsDpad { get; set; } = true;
            public double StickDeadzone { get; set; } = 0.5;

            public bool LeftStickIsAnalog { get; set; }

            public double AnalogDeadzone { get; set; } = 0.1;

            public bool IsPressed(PadButton button, int player)
            {
                if (_owner.PlayerPad(player) is not { } pad) return false;

                if (AnalogStickAsDpad && !LeftStickIsAnalog && StickDirectionPressed(pad, button)) return true;

                // A trigger is an axis to SDL, so L2 and R2 are its press past half its travel.
                if (button is PadButton.L2 or PadButton.R2 && Axis(button == PadButton.L2 ? PadAxis.LeftTrigger : PadAxis.RightTrigger, player) >= 0.5) return true;

                if (!_owner.BindingsFor(player).ButtonToPad.TryGetValue(button, out SDL.GamepadButton sdlButton)) return false;

                return pad.IsRawPressed(sdlButton);
            }

            public double Axis(PadAxis axis, int player)
            {
                if (_owner.PlayerPad(player) is not { } pad) return 0;

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

            public SDL.GamepadButton? GetAnyPressedButton()
            {
                if (_pads.Count == 0) return null;

                _devices.Update();
                for (int i = 0; i < FrontendPadCount; i++)
                    foreach (SDL.GamepadButton b in Enum.GetValues<SDL.GamepadButton>())
                    {
                        if (b == SDL.GamepadButton.Invalid || b == SDL.GamepadButton.Count) continue;
                        if (_pads[i].IsRawPressed(b)) return b;
                    }
                return null;
            }

            public void CloseAll()
            {
                foreach (ConnectedPad pad in _pads) pad.Close();
                _pads.Clear();
                // QuitSubSystem, not Quit - see EmuSen_Settings_Reference.md §4.10.
                if (_sdlInitialized) _devices.Quit();
                _sdlInitialized = false;
            }
        }
    }
}
