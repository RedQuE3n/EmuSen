using System;
using System.Collections.Generic;
using System.Diagnostics;
using SDL3;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
{
    // Polls every connected SDL3 gamepad and seats each as a player - see EmuSen_Input.md §4 and §8, and EmuSen_Settings_Reference.md §4.61.
    public partial class GamepadManager : IDisposable
    {
        // Swapped when a ROM for a different console loads - see EmuSen_Input.md §5.1.
        public GamepadBindingMap Bindings { get; set; }

        // Players 2 on; null, or a null answer, gives them player 1's map - see EmuSen_Input.md §8.4.
        public Func<int, GamepadBindingMap?>? PlayerBindings { get; set; }

        public GamepadBindingMap BindingsFor(int player) => player > 1 && PlayerBindings?.Invoke(player) is { } map ? map : Bindings;

        // Which pad is which player - see EmuSen_Input.md §8.
        public PlayerSlots Players { get; }

        // The library's pads, with the seats and bindings kept here - see EmuSen_RustPlatform.md §14.2.
        private readonly Native _native;

        // start: false leaves SDL untouched until Start, which a window calls once it is shown - see EmuSen_Settings_Reference.md §4.42.
        public GamepadManager(GamepadBindingMap bindings, bool start = true, IPadDevices? devices = null) : this(bindings, start, devices, null) { }

        // only, for a test, the SDL pads the library may see.
        internal GamepadManager(GamepadBindingMap bindings, bool start, IPadDevices? devices, IReadOnlyCollection<uint>? only)
        {
            DeviceNative.Up();
            devices ??= new SdlPadDevices();
            Refuse(devices);
            Bindings = bindings;
            Players = new PlayerSlots();
            _native = new Native(this, devices, only);
            if (start) Start();
        }

        // The library reads SDL's pads or its own simulated set; any other IPadDevices has nothing behind it there - see EmuSen_Backend.md §40.3.
        private static void Refuse(IPadDevices devices)
        {
            if (devices is not (SdlPadDevices or SimulatedPads)) throw new ArgumentException($"{devices.GetType().Name} is not a device layer the platform library reads; GamepadManager takes SdlPadDevices or SimulatedPads.", nameof(devices));
        }

        public bool Started => _native.Started;

        // Raised from Poll for each pad plugged in or pulled out after Start; the pads present at Start are not announced.
        public event Action<PadConnection>? PadChanged;

        // Raised at the end of each Poll, so a window that shows the pad reads it on the one poll there is - see EmuSen_Settings_Reference.md §4.81.
        public event Action? Polled;

        // Every open pad, in the order they were opened; the first is the first controller, which need not be player 1's.
        public IReadOnlyList<ConnectedPad> Pads => _native.Pads;

        public ConnectedPad? Primary => Pads.Count > 0 ? Pads[0] : null;

        // ES-DE's "Only accept input from first controller": the interface reads the first pad alone - see EmuSen_Settings_Reference.md §4.61.
        public bool FirstControllerOnly
        {
            get => _native.Flag(DeviceNative.SettingFirstOnly);
            set => _native.Set(DeviceNative.SettingFirstOnly, value ? 1 : 0);
        }

        // How many of Pads, from the first, the interface reads.
        public int FrontendPadCount => _native.FrontendPadCount;

        // Read in place of the device when set, so a test drives the same path a real pad does - see EmuSen_Settings_Reference.md §4.45.
        public SimulatedPad? Simulated
        {
            get => (Devices as SimulatedPads)?.First;
            set => UseDevices(value is null ? new SdlPadDevices() : SimulatedPads.With(value));
        }

        public IPadDevices Devices => _native.Devices;

        // Lets go of every pad on the old devices and starts on the new ones.
        public void UseDevices(IPadDevices devices)
        {
            Refuse(devices);
            _native.UseDevices(devices);
        }

        // Idempotent, and on the thread that polls, as SDL asks.
        public void Start()
        {
            _native.Start();
        }

        // When the last rescan was, on the manager's own clock; a second before it starts, so the first poll rescans.
        internal TimeSpan LastRescan => _native.LastRescan;

        // Call once per frame tick: lets go of pads pulled out, and opens new ones on SDL's events or the one-a-second rescan - see EmuSen_Settings_Reference.md §4.4 and §4.61.
        public void Poll()
        {
            _native.Poll();
        }

        // The player chose a seat for the pad, 0 for none; the pads that light a number show the new ones.
        public void Assign(ConnectedPad pad, int player)
        {
            Players.Move(pad, player);
            _arranged = true;
            LightPlayers();
        }

        private void LightPlayers()
        {
            _native.LightPlayers();
        }

        // The pad the game hears as the player; with the first controller alone, player 1's only - see EmuSen_Input.md §8.3.
        public ConnectedPad? PlayerPad(int player) => FirstControllerOnly && player > 1 ? null : Players.PadFor(player);

        // The player's pad's motors, strong and weak, as the game drives them this frame - see EmuSen_Input.md §9.
        public void Rumble(int player, ushort strong, ushort weak) => PlayerPad(player)?.Rumble(strong, weak);

        // Stick-as-d-pad and its threshold - see EmuSen_Settings_Reference.md §4.4.
        public bool AnalogStickAsDpad
        {
            get => _native.Flag(DeviceNative.SettingStickAsDpad);
            set => _native.Set(DeviceNative.SettingStickAsDpad, value ? 1 : 0);
        }

        public double StickDeadzone
        {
            get => _native.Get(DeviceNative.SettingStickDeadzone);
            set => _native.Set(DeviceNative.SettingStickDeadzone, value);
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
            get => _native.Flag(DeviceNative.SettingLeftStickAnalog);
            set => _native.Set(DeviceNative.SettingLeftStickAnalog, value ? 1 : 0);
        }

        // Below this share of its travel an axis reads zero, so a pad at rest does not drift - see EmuSen_Input.md §7.3.
        public double AnalogDeadzone
        {
            get => _native.Get(DeviceNative.SettingAnalogDeadzone);
            set => _native.Set(DeviceNative.SettingAnalogDeadzone, value);
        }

        public bool IsPressed(PadButton button) => IsPressed(button, 1);

        // Through the player's own bindings, on the player's own pad.
        public bool IsPressed(PadButton button, int player) => _native.IsPressed(button, player);

        // Sticks -1 to 1 with right and down positive, as SDL and the RetroPad have them, and triggers 0 to 1 - see EmuSen_Input.md §7.
        public double Axis(PadAxis axis) => Axis(axis, 1);

        public double Axis(PadAxis axis, int player) => _native.Axis(axis, player);

        // Player 1's pad's own buttons and axes, whatever a console's bindings say - see EmuSen_Settings_Reference.md §4.29.
        public bool IsRawPressed(SDL.GamepadButton button) => Primary?.IsRawPressed(button) ?? false;

        public double RawAxis(SDL.GamepadAxis axis) => Primary?.RawAxis(axis) ?? 0;

        // First button held on any pad the interface reads, for the bindings page's capture - see EmuSen_Input.md §10.5.
        public SDL.GamepadButton? GetAnyPressedButton() => _native.GetAnyPressedButton();

        public void Dispose()
        {
            _native.CloseAll();
        }

        // The library's pads behind this manager: what it opened and closed is seated, lit and announced here.
        private sealed unsafe class Native
        {
            private readonly GamepadManager _owner;
            private readonly NativeHandle _handle;
            private readonly List<ConnectedPad> _pads = new();
            // A test's own list of the SDL pads the library may see, read again at each poll as the test attaches more.
            private readonly IReadOnlyCollection<uint>? _only;

            // SDL's pads, or a simulated set that is the library's own.
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
                        int player = _owner.SeatAhead(pad, _owner.Players.Seat(pad));
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

            // One crossing when the bindings are a map to look in; a player's own bindings are a delegate, asked only when the stick and the trigger do not answer.
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


            private long Number(uint which)
            {
                long value = DeviceNative.PadsGet(_handle.Value, 0, which, 0);
                GC.KeepAlive(_handle);
                return value;
            }
        }
    }
}
