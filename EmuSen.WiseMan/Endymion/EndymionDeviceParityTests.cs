using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Endymion
{
    // Endymion's C# devices and the platform library's, driven alike on simulated pads, SDL's virtual pads and SDL's dummy audio, and compared - see EmuSen_RustPlatform.md §14.5.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed unsafe class EndymionDeviceParityTests
    {
        public EndymionDeviceParityTests() => Assert.True(DeviceNative.Ready, DeviceNative.Report);

        private static readonly SDL.GamepadButton[] SdlButtons = Enum.GetValues<SDL.GamepadButton>().Where(b => b != SDL.GamepadButton.Invalid && b != SDL.GamepadButton.Count).ToArray();
        private static readonly SDL.GamepadAxis[] SdlAxes = Enum.GetValues<SDL.GamepadAxis>().Where(a => a != SDL.GamepadAxis.Invalid && a != SDL.GamepadAxis.Count).ToArray();
        private static readonly PadButton[] Buttons = Enum.GetValues<PadButton>();
        private static readonly PadAxis[] Axes = Enum.GetValues<PadAxis>();
        private static readonly double[] Readings = { 0, 1, -1, 0.5, -0.5, 0.49, 0.51, 0.05, 0.1, 0.0999, 0.95, 2, -2, double.NaN, 1.5 / 32767, 2.5 / 32767, double.PositiveInfinity };

        private static string Bits(double value) => BitConverter.DoubleToInt64Bits(value).ToString("X16");

        private static void AssertNone(List<string> different, string what) =>
            Assert.True(different.Count == 0, $"{different.Count} {what} differ:\n{string.Join("\n", different.Take(12))}");

        private static string Outcome(Func<string> call)
        {
            try { return call(); }
            catch (Exception e) { return $"{e.GetType().Name}: {e.Message}"; }
        }

        [Fact]
        public void The_device_half_is_what_the_variable_says_and_calls_the_sdl_the_csharp_calls()
        {
            Assert.Equal(Environment.GetEnvironmentVariable(EndymionNative.Variable) == "1", DeviceNative.Active);
            string value = Guid.NewGuid().ToString("N");
            Assert.Null(DeviceNative.Hint("EMUSEN_PARITY_PROBE"));
            Assert.True(SDL.SetHint("EMUSEN_PARITY_PROBE", value));
            Assert.Equal(value, DeviceNative.Hint("EMUSEN_PARITY_PROBE"));
            SDL.ResetHint("EMUSEN_PARITY_PROBE");
            Assert.Null(DeviceNative.Hint("EMUSEN_PARITY_PROBE"));
        }

        // A pad's whole state as a test can read it.
        private static string State(SimulatedPad pad) =>
            $"{pad.Name}|{pad.Guid ?? "null"}|{pad.Path ?? "null"}|{pad.PlayerIndex}|{pad.Type}|{string.Concat(SdlButtons.Select(b => pad.IsHeld(b) ? '1' : '0'))}|{string.Join(",", SdlAxes.Select(pad.Axis))}";

        // One random thing done to a pad and its twin.
        private static string Touch(Random random, SimulatedPad a, SimulatedPad b)
        {
            switch (random.Next(9))
            {
                case 0: SDL.GamepadButton press = SdlButtons[random.Next(SdlButtons.Length)]; a.Press(press); b.Press(press); return $"press {press}";
                case 1: SDL.GamepadButton release = SdlButtons[random.Next(SdlButtons.Length)]; a.Release(release); b.Release(release); return $"release {release}";
                case 2:
                    SDL.GamepadAxis axis = SdlAxes[random.Next(SdlAxes.Length)];
                    double value = random.Next(3) == 0 ? Readings[random.Next(Readings.Length)] : (random.NextDouble() - 0.5) * 2.2;
                    a.SetAxis(axis, value); b.SetAxis(axis, value);
                    return $"axis {axis} {value}";
                case 3: a.ReleaseAll(); b.ReleaseAll(); return "release all";
                case 4: string name = new[] { "First", "Second", "", "Pad é", "日本のパッド", "Simulated pad" }[random.Next(6)]; a.Name = name; b.Name = name; return $"name {name}";
                case 5: string? guid = new[] { null, "", "shared", "0300000009120000" }[random.Next(4)]; a.Guid = guid; b.Guid = guid; return $"guid {guid}";
                case 6: string? path = new[] { null, "", "/dev/input/a", "/dev/input/b" }[random.Next(4)]; a.Path = path; b.Path = path; return $"path {path}";
                case 7: var type = (SDL.GamepadType)random.Next(0, 12); a.Type = type; b.Type = type; return $"type {type}";
                default: int index = random.Next(-1, 8); a.PlayerIndex = index; b.PlayerIndex = index; return $"index {index}";
            }
        }

        [Fact]
        public void A_simulated_pad_keeps_what_a_test_does_to_it_the_same_way()
        {
            var random = new Random(61);
            var different = new List<string>();
            for (int run = 0; run < 200; run++)
            {
                SimulatedPad managed = new(native: false), native = new(native: true);
                Assert.True(native.IsNative && !managed.IsNative);
                // Each took the next path of the one counter, so only their numbers differ.
                Assert.StartsWith("/dev/input/simulated", native.Path);
                native.Path = managed.Path;
                for (int step = 0; step < 40; step++)
                {
                    string what = Touch(random, managed, native);
                    if (State(managed) != State(native)) different.Add($"run {run} step {step} {what}: {State(managed)} and {State(native)}");
                }
            }
            AssertNone(different, "pads");
            Assert.Equal(nameof(ArgumentException), Outcome(() => { new SimulatedPads(native: true).Connect(new SimulatedPad(native: false)); return ""; }).Split(':')[0]);
            Assert.Equal(nameof(ArgumentException), Outcome(() => { new SimulatedPads(native: false).Connect(new SimulatedPad(native: true)); return ""; }).Split(':')[0]);
        }

        // The set as a device layer answers: everything a manager could ask of every handle it was ever given.
        private static string Asked(SimulatedPads set, List<IntPtr> handles, List<SimulatedPad> twins) =>
            $"first {twins.IndexOf(set.First!)} open {set.OpenHandles} opens {set.Opens} closes {set.Closes} init {set.Initialized} attached [{string.Join(",", set.Attached())}] " +
            string.Join(" ", handles.Select(h => $"{h:X}:{set.IsAttached(h)},{set.Name(h) ?? "null"},{set.Guid(h)},{set.Path(h) ?? "null"},{set.Type(h)},{set.Label(h, SDL.GamepadButton.South)},{set.Button(h, SDL.GamepadButton.South)},{set.Axis(h, SDL.GamepadAxis.LeftX)}"));

        [Fact]
        public void A_simulated_set_is_the_same_device_layer_to_a_manager_that_asks_it()
        {
            var random = new Random(62);
            var different = new List<string>();
            for (int run = 0; run < 120; run++)
            {
                SimulatedPads managed = new(native: false), native = new(native: true);
                List<SimulatedPad> managedPads = new(), nativePads = new();
                List<IntPtr> handles = new() { IntPtr.Zero, new IntPtr(0x999) };
                for (int step = 0; step < 50; step++)
                {
                    string what;
                    switch (random.Next(10))
                    {
                        case 0:
                        case 1:
                            SimulatedPad a = new(native: false), b = new(native: true);
                            b.Path = a.Path;
                            managedPads.Add(a); nativePads.Add(b);
                            what = $"connect {managed.Connect(a)} {native.Connect(b)}";
                            break;
                        case 2 when managedPads.Count > 0:
                            int gone = random.Next(managedPads.Count);
                            managed.Disconnect(managedPads[gone]); native.Disconnect(nativePads[gone]);
                            what = $"disconnect {gone}";
                            break;
                        case 3 when managedPads.Count > 0:
                            int again = random.Next(managedPads.Count);
                            what = $"connect again {managed.Connect(managedPads[again])} {native.Connect(nativePads[again])}";
                            break;
                        case 4:
                            uint id = (uint)random.Next(0, 8);
                            IntPtr h = managed.Open(id), n = native.Open(id);
                            what = $"open {id}: {h:X} {n:X}";
                            if (h != n) different.Add($"run {run} step {step} {what}");
                            if (h != IntPtr.Zero) handles.Add(h);
                            break;
                        case 5:
                            IntPtr close = handles[random.Next(handles.Count)];
                            managed.Close(close); native.Close(close);
                            what = $"close {close:X}";
                            break;
                        case 6: what = $"changed {managed.DevicesChanged()} {native.DevicesChanged()}"; if (!what.EndsWith("True True") && !what.EndsWith("False False")) different.Add($"run {run} step {step} {what}"); break;
                        case 7: what = random.Next(2) == 0 ? $"init {managed.Init()} {native.Init()}" : "quit"; if (what == "quit") { managed.Quit(); native.Quit(); } break;
                        case 8:
                            IntPtr lit = handles[random.Next(handles.Count)];
                            int index = random.Next(-1, 8);
                            managed.SetPlayerIndex(lit, index); native.SetPlayerIndex(lit, index);
                            what = $"light {lit:X} {index}";
                            break;
                        default:
                            if (managedPads.Count == 0) { what = "nothing"; break; }
                            int touched = random.Next(managedPads.Count);
                            what = Touch(random, managedPads[touched], nativePads[touched]);
                            break;
                    }
                    managed.Update(); native.Update();
                    string m = Asked(managed, handles, managedPads), v = Asked(native, handles, nativePads);
                    if (m != v) different.Add($"run {run} step {step} {what}:\n  {m}\n  {v}");
                    string lights = string.Join(",", managedPads.Select(p => p.PlayerIndex)), nativeLights = string.Join(",", nativePads.Select(p => p.PlayerIndex));
                    if (lights != nativeLights) different.Add($"run {run} step {step} {what}: lights {lights} and {nativeLights}");
                }
            }
            AssertNone(different, "sets");
        }

        // A manager on one implementation with everything a comparison reads of it.
        private sealed class Side
        {
            public readonly bool Native;
            public GamepadManager Manager;
            public SimulatedPads Set;
            public readonly List<SimulatedPad> Twins = new();
            public readonly List<ConnectedPad> Pool = new();
            public readonly List<string> Log = new();
            public int Polls;

            public Side(bool native, bool start)
            {
                Native = native;
                Set = new SimulatedPads(native);
                Manager = Make(start);
            }

            public GamepadManager Make(bool start)
            {
                var manager = new GamepadManager(new GamepadBindingMap(), start, Set, Native);
                manager.PadChanged += c => Log.Add($"{Index(c.Pad)} {c.Connected} player {c.Player} {c.Pad.Name} open {c.Pad.IsOpen} pads {manager.Pads.Count}");
                manager.Polled += () => Polls++;
                return manager;
            }

            // A pad by the order this side first saw it, which is the same order on both.
            public int Index(ConnectedPad pad)
            {
                if (!Pool.Contains(pad)) Pool.Add(pad);
                return Pool.IndexOf(pad);
            }

            public SimulatedPad Add()
            {
                var pad = new SimulatedPad(Native);
                Twins.Add(pad);
                return pad;
            }

            public string Dump()
            {
                GamepadManager m = Manager;
                foreach (ConnectedPad pad in m.Pads) Index(pad);
                var text = new System.Text.StringBuilder();
                text.Append($"started {m.Started} connected {m.IsConnected} front {m.FrontendPadCount} name {m.ControllerName ?? "null"} type {m.ControllerType} any {m.GetAnyPressedButton()?.ToString() ?? "null"} label {m.ButtonLabel(SDL.GamepadButton.South) ?? "null"} ");
                text.Append($"first {m.FirstControllerOnly} dpad {m.AnalogStickAsDpad} {Bits(m.StickDeadzone)} analog {m.LeftStickIsAnalog} {Bits(m.AnalogDeadzone)} simulated {Twins.IndexOf(m.Simulated!)} polls {Polls}\n");
                text.Append("pads " + string.Join(" ", m.Pads.Select(p => $"{Index(p)}:{p.Id},{p.Name},{p.Guid},{p.Path ?? "null"},{p.IsOpen},{p.Type},{m.Players.PlayerOf(p)}")) + "\n");
                text.Append("pool " + string.Join(" ", Pool.Select(p => $"{p.Name},{p.IsOpen},{p.Type},{p.IsRawPressed(SDL.GamepadButton.South)},{Bits(p.RawAxis(SDL.GamepadAxis.LeftX))},{p.ButtonLabel(SDL.GamepadButton.East) ?? "null"}")) + "\n");
                text.Append("seats " + string.Join(" ", Enumerable.Range(1, PlayerSlots.MaxPlayers).Select(p => m.Players.SeatOf(p) is { } pad ? $"{Index(pad)}" : "-")) + $" highest {m.Players.Highest}\n");
                for (int player = 0; player <= PlayerSlots.MaxPlayers + 1; player++)
                    text.Append($"p{player} {string.Concat(Buttons.Select(b => m.IsPressed(b, player) ? '1' : '0'))} {string.Join(",", Axes.Select(a => Bits(m.Axis(a, player))))} pad {(m.PlayerPad(player) is { } pp ? Index(pp) : -1)}\n");
                text.Append($"raw {string.Concat(SdlButtons.Select(b => m.IsRawPressed(b) ? '1' : '0'))} {string.Join(",", SdlAxes.Select(a => Bits(m.RawAxis(a))))}\n");
                text.Append($"set open {Set.OpenHandles} opens {Set.Opens} closes {Set.Closes} init {Set.Initialized} attached [{string.Join(",", Set.Attached())}] lights {string.Join(",", Twins.Select(t => t.PlayerIndex))}\n");
                text.Append("log " + string.Join(" / ", Log));
                return text.ToString();
            }
        }

        [Fact]
        public void A_manager_opens_seats_lights_announces_and_reads_its_pads_the_same_way()
        {
            var random = new Random(63);
            var different = new List<string>();
            for (int run = 0; run < 150 && different.Count < 5; run++)
            {
                bool start = random.Next(5) != 0;
                Side managed = new(false, start), native = new(true, start);
                void Both(Action<Side> act) { act(managed); act(native); }
                for (int i = random.Next(0, 4); i > 0; i--) Both(s => { SimulatedPad pad = s.Add(); pad.Path = $"/dev/input/start{i}"; s.Set.Connect(pad); });
                if (random.Next(3) == 0) Both(s => { s.Manager.Dispose(); s.Manager = s.Make(start); });
                var history = new List<string>();
                for (int step = 0; step < 70; step++)
                {
                    string what = "";
                    switch (random.Next(18))
                    {
                        case 0:
                            string name = new[] { "Xbox Controller", "DualSense", "Pro Controller", "" }[random.Next(4)];
                            string? guid = random.Next(4) == 0 ? (random.Next(2) == 0 ? "shared" : "") : null;
                            string path = $"/dev/input/run{run}step{(random.Next(3) == 0 ? 0 : step)}";
                            var type = (SDL.GamepadType)random.Next(0, 8);
                            Both(s => { SimulatedPad pad = s.Add(); pad.Name = name; pad.Guid = guid; pad.Path = path; pad.Type = type; s.Set.Connect(pad); });
                            what = $"connect {name} {guid} {path}";
                            break;
                        case 1 when managed.Twins.Count > 0:
                            int gone = random.Next(managed.Twins.Count);
                            Both(s => s.Set.Disconnect(s.Twins[gone]));
                            what = $"disconnect {gone}";
                            break;
                        case 2 when managed.Twins.Count > 0:
                            int back = random.Next(managed.Twins.Count);
                            Both(s => s.Set.Connect(s.Twins[back]));
                            what = $"connect again {back}";
                            break;
                        case 3: case 4: case 5: Both(s => s.Manager.Poll()); what = "poll"; break;
                        case 6: case 7: case 8: case 9:
                            if (managed.Twins.Count == 0) break;
                            int touched = random.Next(managed.Twins.Count);
                            int seed = random.Next();
                            Both(s => Touch(new Random(seed), s.Twins[touched], s.Twins[touched]));
                            what = $"touch {touched}: {Touch(new Random(seed), new SimulatedPad(false), new SimulatedPad(false))}";
                            break;
                        case 10 when managed.Pool.Count > 0:
                            int moved = random.Next(managed.Pool.Count), seat = random.Next(-1, 10);
                            string movedManaged = Outcome(() => { managed.Manager.Assign(managed.Pool[moved], seat); return "ok"; }), movedNative = Outcome(() => { native.Manager.Assign(native.Pool[moved], seat); return "ok"; });
                            what = $"assign {moved} to {seat}: {movedManaged}";
                            if (movedManaged != movedNative) different.Add($"run {run} step {step} {what} and {movedNative}");
                            break;
                        case 11: int forgot = random.Next(0, 10); Both(s => s.Manager.Players.Forget(forgot)); what = $"forget {forgot}"; break;
                        case 12:
                            int setting = random.Next(5);
                            double value = Readings[random.Next(Readings.Length)];
                            Both(s =>
                            {
                                switch (setting)
                                {
                                    case 0: s.Manager.FirstControllerOnly ^= true; break;
                                    case 1: s.Manager.AnalogStickAsDpad ^= true; break;
                                    case 2: s.Manager.LeftStickIsAnalog ^= true; break;
                                    case 3: s.Manager.StickDeadzone = value; break;
                                    default: s.Manager.AnalogDeadzone = value; break;
                                }
                            });
                            what = $"setting {setting} {value}";
                            break;
                        case 13:
                            PadButton rebound = Buttons[random.Next(Buttons.Length)];
                            SDL.GamepadButton to = SdlButtons[random.Next(SdlButtons.Length)];
                            bool unbind = random.Next(4) == 0, second = random.Next(3) == 0;
                            Both(s =>
                            {
                                var map = new GamepadBindingMap();
                                if (unbind) map.Unbind(rebound); else map.Rebind(rebound, to);
                                if (second) s.Manager.PlayerBindings = p => p == 2 ? map : null;
                                else s.Manager.Bindings = map;
                            });
                            what = $"bind {rebound} {(unbind ? "nothing" : to.ToString())} {(second ? "player 2" : "all")}";
                            break;
                        case 14:
                            int[] kept = Enumerable.Range(0, managed.Twins.Count).Where(_ => random.Next(2) == 0).ToArray();
                            Both(s =>
                            {
                                s.Set = new SimulatedPads(s.Native);
                                foreach (int k in kept) s.Set.Connect(s.Twins[k]);
                                s.Manager.UseDevices(s.Set);
                            });
                            what = $"use devices [{string.Join(",", kept)}]";
                            break;
                        case 15: Both(s => s.Manager.Start()); what = "start"; break;
                        case 16 when random.Next(4) == 0: Both(s => s.Manager.Dispose()); what = "dispose"; break;
                        case 17 when managed.Twins.Count > 0 && random.Next(4) == 0:
                            int only = random.Next(managed.Twins.Count);
                            Both(s => { s.Manager.Simulated = s.Twins[only]; s.Set = (SimulatedPads)s.Manager.Devices; });
                            what = $"simulated {only}";
                            break;
                    }
                    history.Add(what);
                    string m = managed.Dump(), v = native.Dump();
                    if (m != v)
                    {
                        different.Add($"run {run} step {step} after [{string.Join("; ", history.TakeLast(8))}]:\n{m}\n--\n{v}");
                        break;
                    }
                }
                Both(s => s.Manager.Dispose());
                if (managed.Dump() != native.Dump()) different.Add($"run {run} disposed:\n{managed.Dump()}\n--\n{native.Dump()}");
            }
            AssertNone(different, "managers");
        }

        // The comparisons a random walk seldom lands on: a reading exactly at a deadzone, a stick one step either side of its threshold, and a deadzone outside its range.
        [Fact]
        public void A_reading_at_the_edge_of_a_deadzone_or_a_threshold_falls_the_same_side()
        {
            var different = new List<string>();
            Side managed = new(false, true), native = new(true, true);
            void Both(Action<Side> act) { act(managed); act(native); }
            Both(s => { s.Set.Connect(s.Add()); s.Manager.Poll(); });
            // The poll took the devices' word that a pad was plugged in, so there is none left to take.
            Assert.Equal((false, false), (managed.Set.DevicesChanged(), native.Set.DevicesChanged()));
            int[] steps = { 0, 1, 2, 1637, 1638, 1639, 3276, 3277, 16383, 16384, 31128, 31129, 32766, 32767 };
            double[] deadzones = { 0, -1, 0.05, 0.1, 0.5, 0.95, 1, 2, double.NaN, 1638 / 32767.0, 16384 / 32767.0, 3277 / 32767.0, 1 / 32767.0 };
            foreach (double deadzone in deadzones)
                foreach (int magnitude in steps)
                    foreach (int sign in new[] { 1, -1 })
                    {
                        double value = sign * magnitude / 32767.0;
                        Both(s =>
                        {
                            s.Manager.StickDeadzone = deadzone;
                            s.Manager.AnalogDeadzone = deadzone;
                            foreach (SDL.GamepadAxis axis in SdlAxes) s.Twins[0].SetAxis(axis, value);
                        });
                        string Read(Side s) =>
                            $"{s.Twins[0].Axis(SDL.GamepadAxis.LeftX)} {string.Concat(Buttons.Select(b => s.Manager.IsPressed(b) ? '1' : '0'))} {string.Join(",", Axes.Select(a => Bits(s.Manager.Axis(a))))} {Bits(s.Manager.RawAxis(SDL.GamepadAxis.LeftY))}";
                        if (Read(managed) != Read(native)) different.Add($"deadzone {deadzone} reading {value}: {Read(managed)} and {Read(native)}");
                    }
            AssertNone(different, "readings");
            Both(s => s.Manager.Dispose());
        }

        [Fact]
        public void A_rescan_is_due_at_the_same_ticks_and_first_at_once()
        {
            var random = new Random(64);
            var different = new List<string>();
            long[] edges = { 0, 1, -1, 9_999_999, 10_000_000, 10_000_001, -10_000_000, TimeSpan.FromDays(1).Ticks, long.MaxValue / 4, long.MinValue / 4 };
            for (int i = 0; i < 20_000; i++)
            {
                long now = random.Next(3) == 0 ? edges[random.Next(edges.Length)] : random.NextInt64(-50_000_000, 50_000_000);
                long last = random.Next(3) == 0 ? edges[random.Next(edges.Length)] : now - random.NextInt64(-20_000_000, 20_000_000);
                bool managed = (bool)typeof(GamepadManager).GetMethod("RescanDue", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)!.Invoke(null, [TimeSpan.FromTicks(now), TimeSpan.FromTicks(last)])!;
                if (managed != (DeviceNative.PadsRescanDue(now, last) != 0)) different.Add($"{now} after {last}: {managed}");
            }
            AssertNone(different, "times");
            var set = new SimulatedPads(native: true);
            Assert.Equal(TimeSpan.FromSeconds(-1), new GamepadManager(new GamepadBindingMap(), start: false, set, native: true).LastRescan);
            Assert.Equal(TimeSpan.FromSeconds(-1), new GamepadManager(new GamepadBindingMap(), start: false, new SimulatedPads(native: false), native: false).LastRescan);
            // A difference past what a tick count holds, for which the C# throws, is a rescan to the library.
            Assert.Equal(1, DeviceNative.PadsRescanDue(long.MaxValue, long.MinValue));
        }

        // A device layer the library cannot know: the C# set behind a wrapper.
        private sealed class Wrapped(SimulatedPads inner) : IPadDevices
        {
            public bool Init() => inner.Init();
            public void Quit() => inner.Quit();
            public uint[] Attached() => inner.Attached();
            public IntPtr Open(uint id) => inner.Open(id);
            public void Close(IntPtr pad) => inner.Close(pad);
            public bool IsAttached(IntPtr pad) => inner.IsAttached(pad);
            public bool DevicesChanged() => inner.DevicesChanged();
            public void Update() => inner.Update();
            public bool Button(IntPtr pad, SDL.GamepadButton button) => inner.Button(pad, button);
            public short Axis(IntPtr pad, SDL.GamepadAxis axis) => inner.Axis(pad, axis);
            public string? Name(IntPtr pad) => inner.Name(pad);
            public SDL.GamepadType Type(IntPtr pad) => inner.Type(pad);
            public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) => inner.Label(pad, button);
            public string Guid(IntPtr pad) => inner.Guid(pad);
            public string? Path(IntPtr pad) => inner.Path(pad);
            public void SetPlayerIndex(IntPtr pad, int index) => inner.SetPlayerIndex(pad, index);
        }

        [Fact]
        public void Devices_the_library_does_not_know_run_the_csharp_manager_with_the_settings_it_had()
        {
            foreach (bool nativeSet in new[] { false, true })
            {
                var pad = new SimulatedPad(nativeSet) { Name = "Wrapped" };
                var inner = new SimulatedPads(nativeSet);
                inner.Connect(pad);
                using var manager = new GamepadManager(new GamepadBindingMap(), start: true, new Wrapped(inner), native: true);
                Assert.Equal("Wrapped", manager.ControllerName);
                pad.Press(SDL.GamepadButton.South);
                Assert.True(manager.IsPressed(PadButton.B));
            }

            var first = new SimulatedPad(native: true) { Name = "Library" };
            var known = new SimulatedPads(native: true);
            known.Connect(first);
            using var moved = new GamepadManager(new GamepadBindingMap(), start: true, known, native: true)
            {
                FirstControllerOnly = true, AnalogStickAsDpad = false, StickDeadzone = 0.25, LeftStickIsAnalog = true, AnalogDeadzone = 0.3,
            };
            Assert.Equal(1, known.OpenHandles);
            var second = new SimulatedPad(native: false) { Name = "CSharp" };
            var unknown = new SimulatedPads(native: false);
            unknown.Connect(second);
            moved.UseDevices(new Wrapped(unknown));
            Assert.Equal((0, false), (known.OpenHandles, known.Initialized));
            Assert.Equal(("CSharp", 1), (moved.ControllerName, moved.Players.PlayerOf(moved.Pads[0])));
            Assert.Equal((true, false, 0.25, true, 0.3), (moved.FirstControllerOnly, moved.AnalogStickAsDpad, moved.StickDeadzone, moved.LeftStickIsAnalog, moved.AnalogDeadzone));
        }

        // SDL's virtual joysticks under each manager in turn, the library's seeing only these, so no pad on the desk is opened by either.
        private static string OnVirtualPads(bool native)
        {
            var mine = new HashSet<uint>();
            var joysticks = new Dictionary<uint, IntPtr>();
            var names = new List<IntPtr>();
            var log = new List<string>();
            Assert.True(SDL.InitSubSystem(SDL.InitFlags.Gamepad), SDL.GetError());
            try
            {
                uint Attach(string name, ushort product)
                {
                    IntPtr text = Marshal.StringToHGlobalAnsi(name);
                    names.Add(text);
                    var desc = new SDL.VirtualJoystickDesc
                    {
                        Version = (uint)Marshal.SizeOf<SDL.VirtualJoystickDesc>(), Type = SDL.JoystickType.Gamepad, NAxes = (ushort)SDL.GamepadAxis.Count,
                        NButtons = (ushort)SDL.GamepadButton.Count, VendorID = 0x1209, ProductID = product, Name = text,
                    };
                    uint id = SDL.AttachVirtualJoystick(in desc);
                    Assert.True(id != 0, SDL.GetError());
                    mine.Add(id);
                    joysticks[id] = SDL.OpenJoystick(id);
                    return id;
                }
                void Detach(uint id)
                {
                    SDL.CloseJoystick(joysticks[id]);
                    joysticks.Remove(id);
                    Assert.True(SDL.DetachVirtualJoystick(id), SDL.GetError());
                    mine.Remove(id);
                }

                uint a = Attach("EmuSen Parity Pad", 0x0011), b = Attach("EmuSen Parity Pad", 0x0011), c = Attach("EmuSen Other Parity Pad", 0x0012);
                IPadDevices devices = native ? new SdlPadDevices() : new VirtualOnly(new SdlPadDevices(), mine);
                using var manager = new GamepadManager(new GamepadBindingMap(), start: true, devices, native, native ? mine : null);
                var order = new List<ConnectedPad>();
                manager.PadChanged += change =>
                {
                    if (!order.Contains(change.Pad)) order.Add(change.Pad);
                    log.Add($"changed {order.IndexOf(change.Pad)} {change.Connected} player {change.Player} {change.Pad.Name} open {change.Pad.IsOpen}");
                };
                void Poll(int wanted)
                {
                    for (int i = 0; i < 40; i++)
                    {
                        SDL.PumpEvents();
                        manager.Poll();
                        if (manager.Pads.Count == wanted) break;
                        System.Threading.Thread.Sleep(50);
                    }
                    foreach (ConnectedPad pad in manager.Pads)
                        if (!order.Contains(pad)) order.Add(pad);
                    // An id is SDL's own count of every joystick the process has attached, so a pad is told by its place and not its id.
                    log.Add("pads " + string.Join(" ", manager.Pads.Select(p => $"{order.IndexOf(p)}:{p.Name},{p.Guid},{p.Path ?? "null"},{p.Type},{p.IsOpen},{manager.Players.PlayerOf(p)},{p.ButtonLabel(SDL.GamepadButton.South) ?? "null"}")));
                    log.Add("held " + string.Join(" ", Enumerable.Range(1, 4).Select(p => string.Concat(Buttons.Select(btn => manager.IsPressed(btn, p) ? '1' : '0')) + ":" + string.Join(",", Axes.Select(ax => Bits(manager.Axis(ax, p)))))));
                    log.Add($"any {manager.GetAnyPressedButton()?.ToString() ?? "null"} name {manager.ControllerName} type {manager.ControllerType} seats {string.Join(",", Enumerable.Range(1, 4).Select(p => manager.Players.SeatOf(p) is { } s ? $"{order.IndexOf(s)}{(s.IsOpen ? "" : "x")}" : "-"))}");
                }

                Poll(3);
                Assert.Equal(3, manager.Pads.Count);
                SDL.SetJoystickVirtualButton(joysticks[b], (int)SDL.GamepadButton.South, true);
                SDL.SetJoystickVirtualButton(joysticks[c], (int)SDL.GamepadButton.Start, true);
                SDL.SetJoystickVirtualAxis(joysticks[a], (int)SDL.GamepadAxis.LeftX, -20000);
                SDL.SetJoystickVirtualAxis(joysticks[c], (int)SDL.GamepadAxis.RightTrigger, 30000);
                Poll(3);
                Detach(b);
                Poll(2);
                Assert.Equal(2, manager.Pads.Count);
                uint replacement = Attach("EmuSen Parity Pad", 0x0011);
                Poll(3);
                Assert.Equal(3, manager.Pads.Count);
                SDL.SetJoystickVirtualButton(joysticks[replacement], (int)SDL.GamepadButton.North, true);
                Poll(3);
                manager.Dispose();
                log.Add($"disposed {manager.Pads.Count} {string.Join(",", order.Select(p => $"{p.Name}:{p.IsOpen}:{p.Type}"))}");
            }
            finally
            {
                foreach (IntPtr joystick in joysticks.Values) SDL.CloseJoystick(joystick);
                foreach (uint id in mine.ToArray()) SDL.DetachVirtualJoystick(id);
                foreach (IntPtr name in names) Marshal.FreeHGlobal(name);
                SDL.PumpEvents();
                SDL.FlushEvents((uint)SDL.EventType.GamepadAdded, (uint)SDL.EventType.GamepadRemoved);
                SDL.QuitSubSystem(SDL.InitFlags.Gamepad);
            }
            return string.Join("\n", log);
        }

        // The real layer showing only the pads a test attached, for the C# manager; the library's is told the same ids.
        private sealed class VirtualOnly(SdlPadDevices sdl, HashSet<uint> mine) : IPadDevices
        {
            public bool Init() => sdl.Init();
            public void Quit() => sdl.Quit();
            public uint[] Attached() => sdl.Attached().Where(mine.Contains).ToArray();
            public IntPtr Open(uint id) => sdl.Open(id);
            public void Close(IntPtr pad) => sdl.Close(pad);
            public bool IsAttached(IntPtr pad) => sdl.IsAttached(pad);
            public bool DevicesChanged() => sdl.DevicesChanged();
            public void Update() => sdl.Update();
            public bool Button(IntPtr pad, SDL.GamepadButton button) => sdl.Button(pad, button);
            public short Axis(IntPtr pad, SDL.GamepadAxis axis) => sdl.Axis(pad, axis);
            public string? Name(IntPtr pad) => sdl.Name(pad);
            public SDL.GamepadType Type(IntPtr pad) => sdl.Type(pad);
            public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) => sdl.Label(pad, button);
            public string Guid(IntPtr pad) => sdl.Guid(pad);
            public string? Path(IntPtr pad) => sdl.Path(pad);
            public void SetPlayerIndex(IntPtr pad, int index) => sdl.SetPlayerIndex(pad, index);
        }

        [Fact]
        public void Sdls_virtual_pads_are_opened_read_and_replaced_the_same_way_through_the_librarys_sdl_layer()
        {
            string managed = OnVirtualPads(native: false), native = OnVirtualPads(native: true);
            Assert.Contains("changed", managed);
            Assert.Equal(managed, native);
        }

        // What of a player does not depend on how fast the dummy device drains.
        private static string Steady(AudioPlayer player) =>
            $"available {player.IsAvailable} rate {player.SampleRate} target {player.RateControl.TargetQueuedFrames} deviation {Bits(player.RateControl.MaxDeviation)} volume {BitConverter.SingleToInt32Bits(player.Volume):X8} " +
            $"in {player.RateControl.TotalInputFrames} shed {player.RateControl.SheddingEvents} {player.RateControl.IsShedding}";

        // The last ratio as well, where the call before left it at the nominal one whatever the queue held.
        private static string Reset(AudioPlayer player) => $"{Steady(player)} last {Bits(player.RateControl.LastRatio)}";

        [Fact]
        public void The_games_audio_player_opens_reopens_and_steers_the_dummy_device_the_same_way()
        {
            SDL.SetHint(SDL.Hints.AudioDriver, "dummy");
            var different = new List<string>();
            // Targets no queue here comes near, and one below zero, which sheds from the first sample: nothing compared then turns on how fast the dummy device drains.
            foreach ((int rate, int latency, double deviation, int frames) in new[] { (32000, 256, 0.005, 4096), (44100, 100, 0.02, 1024), (48000, 5000, 0.0, 512), (96000, -5, 0.005, 2048), (48000, 100_000, 0.01, 4096) })
            {
                AudioPlayer managed = new(rate, latency, deviation, frames, native: false), native = new(rate, latency, deviation, frames, native: true);
                Assert.Equal("dummy", SDL.GetCurrentAudioDriver());
                void Same(string after)
                {
                    if (Steady(managed) != Steady(native)) different.Add($"{rate} {latency} after {after}: {Steady(managed)} and {Steady(native)}");
                }
                void Both(string what, Action<AudioPlayer> act) { act(managed); act(native); Same(what); }
                Same("opening");
                foreach (float volume in new[] { 0.5f, -1f, 2f, float.NaN, 0f, 1f, 0.25f }) Both($"volume {volume}", p => p.Volume = volume);
                Both("null", p => p.Submit(null!, rate));
                Both("empty", p => p.Submit(Array.Empty<short>(), rate));
                short[] tone = Enumerable.Range(0, 2 * 800).Select(i => (short)(Math.Sin(i * 0.05) * 8000)).ToArray();
                Both("a tone", p => p.Submit(tone, rate));
                // A new rate with nothing to play reopens the device and resets the rate control, and steers nothing after.
                Both("empty at a new rate", p => p.Submit(Array.Empty<short>(), rate + 1000));
                if (Reset(managed) != Reset(native)) different.Add($"{rate} {latency} after the reopening: {Reset(managed)} and {Reset(native)}");
                Assert.Equal(Bits(managed.RateControl.NominalRatio), Bits(native.RateControl.LastRatio));
                Both("one sample", p => p.Submit(new short[] { 7 }, rate + 1000));
                Both("a rate of zero", p => p.Submit(tone, 0));
                Both("a rate below zero", p => p.Submit(tone, -44100));
                Both("back to the first rate", p => p.Submit(tone, rate));
                foreach (AudioPlayer player in new[] { managed, native }) Assert.InRange(player.QueuedFrames, 0, 4000);
                // The output count follows the queue, which drains in real time: within the deviation of the four tones' 3,200 frames, or nothing while shedding.
                if (managed.IsAvailable)
                    foreach (AudioPlayer player in new[] { managed, native })
                        Assert.InRange(player.RateControl.TotalOutputFrames, latency > 0 ? (long)(3190 * (1 - deviation)) - 8 : 0, latency > 0 ? (long)(3200 * (1 + deviation)) + 8 : 0);
                Both("disposing", p => p.Dispose());
                Assert.Equal((false, 0, 0), (native.IsAvailable, native.SampleRate, native.QueuedFrames));
                Both("a tone after disposing", p => p.Submit(tone, rate));
                Both("volume after disposing", p => p.Volume = 0.75f);
            }
            AssertNone(different, "players");
        }

        [Fact]
        public void The_interface_sounds_decode_to_the_same_bytes_and_one_replaces_another_the_same_way()
        {
            SDL.SetHint(SDL.Hints.AudioDriver, "dummy");
            string folder = Path.Combine(Path.GetTempPath(), "EmuSenDeviceParity", Guid.NewGuid().ToString("N"));
            try
            {
                var files = new List<string>();
                foreach ((double hz, int rate, double seconds) in new[] { (440.0, 22050, 0.1), (330.0, 48000, 0.25), (660.0, 8000, 0.05), (1000.0, 44100, 2.0), (50.0, 11025, 0.0) })
                {
                    string file = Path.Combine(folder, $"sound é {files.Count}.wav");
                    ThemedSession.Wav(file, hz, rate, seconds);
                    files.Add(file);
                }
                File.WriteAllText(Path.Combine(folder, "not.wav"), "not a sound");
                files.Add(Path.Combine(folder, "not.wav"));
                files.Add(Path.Combine(folder, "missing.wav"));
                files.Add(folder);
                files.Add("");
                // SDL's binding hands over a path up to its first NUL, so this one is the first sound again.
                files.Add(files[0] + "\0.missing");
                foreach (string file in files)
                {
                    byte[]? managed = UiSoundPlayer.Managed.Decode(file), native = UiSoundPlayer.DecodeNative(file);
                    Assert.Equal(managed is null, native is null);
                    if (managed is not null) Assert.True(managed.AsSpan().SequenceEqual(native), $"{file}: {managed.Length} and {native!.Length} bytes");
                }
                Assert.True(UiSoundPlayer.DecodeNative(files[3])!.Length > 64 * 1024, "the long sound is past the first buffer, so it was taken");

                UiSoundPlayer managedPlayer = new(native: false), nativePlayer = new(native: true);
                string Seen(UiSoundPlayer p) => $"open {p.IsOpen} volume {BitConverter.SingleToInt32Bits(p.Volume):X8}";
                void Both(Action<UiSoundPlayer> act) { act(managedPlayer); act(nativePlayer); Assert.Equal(Seen(managedPlayer), Seen(nativePlayer)); }
                Both(_ => { });
                // Silent from here, so a run plays nothing if SDL should open a device that is not the dummy one.
                foreach (float volume in new[] { 0.5f, 3f, -3f, float.NaN, 0f }) Both(p => p.Volume = volume);
                Both(p => p.Play(files[6]));
                Both(p => p.Play(files[5]));
                Both(p => p.Play(files[4]));
                Assert.False(nativePlayer.IsOpen, "a sound that is missing, unreadable or empty opens nothing");
                Both(p => p.Preload(files));
                int whole = UiSoundPlayer.Managed.Decode(files[3])!.Length, shorter = UiSoundPlayer.Managed.Decode(files[1])!.Length;
                Both(p => p.Play(files[3]));
                if (managedPlayer.IsOpen)
                {
                    foreach (UiSoundPlayer p in new[] { managedPlayer, nativePlayer }) Assert.InRange(p.Queued, whole / 2, whole);
                    Both(p => p.Play(files[1]));
                    foreach (UiSoundPlayer p in new[] { managedPlayer, nativePlayer }) Assert.InRange(p.Queued, shorter / 2, shorter);
                    byte[] remembered = new byte[48000 * 8];
                    Both(p => p.Remember("a name that is no file", remembered));
                    Both(p => p.Play("a name that is no file"));
                    foreach (UiSoundPlayer p in new[] { managedPlayer, nativePlayer }) Assert.InRange(p.Queued, remembered.Length / 2, remembered.Length);
                    Both(p => p.Remember("nothing", Array.Empty<byte>()));
                    Both(p => p.Play("nothing"));
                    foreach (UiSoundPlayer p in new[] { managedPlayer, nativePlayer }) Assert.InRange(p.Queued, remembered.Length / 4, remembered.Length);
                }
                Both(p => p.Dispose());
                Assert.Equal((false, 0), (nativePlayer.IsOpen, nativePlayer.Queued));
                Both(p => p.Play(files[1]));
                Assert.False(nativePlayer.IsOpen, "a player disposed does not open again");
            }
            finally
            {
                if (Directory.Exists(folder)) Directory.Delete(folder, recursive: true);
            }
        }

        [Fact]
        public void The_library_has_every_export_the_device_half_calls()
        {
            string header = File.ReadAllText(Path.Combine(EmuSen.Galaxia.ConfigRoot.Managed.Directory, "Platform", "include", "emusen_platform.h"));
            foreach (string export in DeviceNative.Exports) Assert.Contains(export + "(", header);
        }
    }
}
