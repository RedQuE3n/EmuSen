using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Endymion
{
    // Endymion's devices held to the answers its library and its C# both gave before the C# was deleted, on simulated pads, SDL's virtual pads and SDL's dummy audio - see EmuSen_Backend.md §40.3.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed unsafe class EndymionDeviceParityTests
    {
        public EndymionDeviceParityTests() => Assert.True(DeviceNative.Ready, DeviceNative.Report);

        private static ParityGolden Golden(string name) => new("EndymionDeviceParity." + name);

        private static readonly SDL.GamepadButton[] SdlButtons = Enum.GetValues<SDL.GamepadButton>().Where(b => b != SDL.GamepadButton.Invalid && b != SDL.GamepadButton.Count).ToArray();
        private static readonly SDL.GamepadAxis[] SdlAxes = Enum.GetValues<SDL.GamepadAxis>().Where(a => a != SDL.GamepadAxis.Invalid && a != SDL.GamepadAxis.Count).ToArray();
        private static readonly PadButton[] Buttons = Enum.GetValues<PadButton>();
        private static readonly PadAxis[] Axes = Enum.GetValues<PadAxis>();
        private static readonly double[] Readings = { 0, 1, -1, 0.5, -0.5, 0.49, 0.51, 0.05, 0.1, 0.0999, 0.95, 2, -2, double.NaN, 1.5 / 32767, 2.5 / 32767, double.PositiveInfinity };

        private static string Bits(double value) => BitConverter.DoubleToInt64Bits(value).ToString("X16");

        private static string Outcome(Func<string> call)
        {
            try { return call(); }
            catch (Exception e) { return $"{e.GetType().Name}: {e.Message}"; }
        }

        [Fact]
        public void The_device_half_calls_the_sdl_the_csharp_calls()
        {
            string value = Guid.NewGuid().ToString("N");
            Assert.Null(DeviceNative.Hint("EMUSEN_PARITY_PROBE"));
            Assert.True(SDL.SetHint("EMUSEN_PARITY_PROBE", value));
            Assert.Equal(value, DeviceNative.Hint("EMUSEN_PARITY_PROBE"));
            SDL.ResetHint("EMUSEN_PARITY_PROBE");
            Assert.Null(DeviceNative.Hint("EMUSEN_PARITY_PROBE"));
        }

        // A pad's whole state as a test can read it.
        private static string State(SimulatedPad pad) =>
            $"{pad.Name}|{pad.Guid ?? "null"}|{pad.Path ?? "null"}|{pad.PlayerIndex}|{pad.Type}|{pad.Vendor:X4}:{pad.Product:X4}|{string.Concat(SdlButtons.Select(b => pad.IsHeld(b) ? '1' : '0'))}|{string.Join(",", SdlAxes.Select(pad.Axis))}";

        // One random thing done to a pad.
        private static string Touch(Random random, SimulatedPad a)
        {
            switch (random.Next(10))
            {
                case 8: (ushort v, ushort p) = new (ushort, ushort)[] { (0, 0), (0x1A86, 0xE310), (0x045E, 0x0B13), (0xFFFF, 0xFFFF) }[random.Next(4)]; a.Vendor = v; a.Product = p; return $"model {v:X4}:{p:X4}";
                case 0: SDL.GamepadButton press = SdlButtons[random.Next(SdlButtons.Length)]; a.Press(press); return $"press {press}";
                case 1: SDL.GamepadButton release = SdlButtons[random.Next(SdlButtons.Length)]; a.Release(release); return $"release {release}";
                case 2:
                    SDL.GamepadAxis axis = SdlAxes[random.Next(SdlAxes.Length)];
                    double value = random.Next(3) == 0 ? Readings[random.Next(Readings.Length)] : (random.NextDouble() - 0.5) * 2.2;
                    a.SetAxis(axis, value);
                    return $"axis {axis} {value}";
                case 3: a.ReleaseAll(); return "release all";
                case 4: string name = new[] { "First", "Second", "", "Pad é", "日本のパッド", "Simulated pad" }[random.Next(6)]; a.Name = name; return $"name {name}";
                case 5: string? guid = new[] { null, "", "shared", "0300000009120000" }[random.Next(4)]; a.Guid = guid; return $"guid {guid}";
                case 6: string? path = new[] { null, "", "/dev/input/a", "/dev/input/b" }[random.Next(4)]; a.Path = path; return $"path {path}";
                case 7: var type = (SDL.GamepadType)random.Next(0, 12); a.Type = type; return $"type {type}";
                default: int index = random.Next(-1, 8); a.PlayerIndex = index; return $"index {index}";
            }
        }

        // A new pad at a path of its own place, since the counter its first path comes from is the whole process's.
        private static SimulatedPad Numbered(int place)
        {
            SimulatedPad pad = new SimulatedPad();
            Assert.StartsWith("/dev/input/simulated", pad.Path);
            pad.Path = $"/dev/input/simulated{place}";
            return pad;
        }

        [Fact]
        public void A_simulated_pad_keeps_what_a_test_does_to_it_as_recorded()
        {
            var golden = Golden("Pad");
            var random = new Random(61);
            for (int run = 0; run < 200; run++)
            {
                SimulatedPad pad = Numbered(0);
                for (int step = 0; step < 40; step++)
                {
                    string what = Touch(random, pad);
                    golden.AddChunked("step", $"{what}: {State(pad)}", 64);
                }
            }
            golden.Check(100);
        }

        // The set as a device layer answers: everything a manager could ask of every handle it was ever given.
        private static string Asked(SimulatedPads set, List<IntPtr> handles, List<SimulatedPad> pads) =>
            $"first {pads.IndexOf(set.First!)} open {set.OpenHandles} opens {set.Opens} closes {set.Closes} init {set.Initialized} attached [{string.Join(",", set.Attached())}] " +
            string.Join(" ", handles.Select(h => $"{h:X}:{set.IsAttached(h)},{set.Name(h) ?? "null"},{set.Guid(h)},{set.Path(h) ?? "null"},{set.VendorProduct(h):X8},{set.Type(h)},{set.Label(h, SDL.GamepadButton.South)},{set.Button(h, SDL.GamepadButton.South)},{set.Axis(h, SDL.GamepadAxis.LeftX)}"));

        [Fact]
        public void A_simulated_set_is_the_recorded_device_layer_to_a_manager_that_asks_it()
        {
            var golden = Golden("Set");
            var random = new Random(62);
            for (int run = 0; run < 120; run++)
            {
                SimulatedPads set = new SimulatedPads();
                List<SimulatedPad> pads = new();
                List<IntPtr> handles = new() { IntPtr.Zero, new IntPtr(0x999) };
                for (int step = 0; step < 50; step++)
                {
                    string what;
                    switch (random.Next(10))
                    {
                        case 0:
                        case 1:
                            SimulatedPad made = Numbered(pads.Count);
                            pads.Add(made);
                            what = $"connect {set.Connect(made)}";
                            break;
                        case 2 when pads.Count > 0:
                            int gone = random.Next(pads.Count);
                            set.Disconnect(pads[gone]);
                            what = $"disconnect {gone}";
                            break;
                        case 3 when pads.Count > 0:
                            int again = random.Next(pads.Count);
                            what = $"connect again {set.Connect(pads[again])}";
                            break;
                        case 4:
                            uint id = (uint)random.Next(0, 8);
                            IntPtr h = set.Open(id);
                            what = $"open {id}: {h:X}";
                            if (h != IntPtr.Zero) handles.Add(h);
                            break;
                        case 5:
                            IntPtr close = handles[random.Next(handles.Count)];
                            set.Close(close);
                            what = $"close {close:X}";
                            break;
                        case 6: what = $"changed {set.DevicesChanged()}"; break;
                        case 7:
                            if (random.Next(2) == 0) what = $"init {set.Init()}";
                            else { set.Quit(); what = "quit"; }
                            break;
                        case 8:
                            IntPtr lit = handles[random.Next(handles.Count)];
                            int index = random.Next(-1, 8);
                            set.SetPlayerIndex(lit, index);
                            what = $"light {lit:X} {index}";
                            break;
                        default:
                            if (pads.Count == 0) { what = "nothing"; break; }
                            what = Touch(random, pads[random.Next(pads.Count)]);
                            break;
                    }
                    set.Update();
                    golden.AddChunked("step", $"{what}: {Asked(set, handles, pads)} lights {string.Join(",", pads.Select(p => p.PlayerIndex))}", 64);
                }
            }
            golden.Check((120 * 50 + 63) / 64); // Every step adds one answer: 6,000 in chunks of 64 make 94 cases.
        }

        // A manager with everything a comparison reads of it.
        private sealed class Side
        {
            public GamepadManager Manager;
            public SimulatedPads Set;
            public readonly List<SimulatedPad> Twins = new();
            public readonly List<ConnectedPad> Pool = new();
            public readonly List<string> Log = new();
            public int Polls;

            public Side(bool start)
            {
                Set = new SimulatedPads();
                Manager = Make(start);
            }

            public GamepadManager Make(bool start)
            {
                GamepadManager manager = new GamepadManager(new GamepadBindingMap(), start, Set);
                manager.PadChanged += c => Log.Add($"{Index(c.Pad)} {c.Connected} player {c.Player} {c.Pad.Name} open {c.Pad.IsOpen} pads {manager.Pads.Count}");
                manager.Polled += () => Polls++;
                return manager;
            }

            // A pad by the order it was first seen.
            public int Index(ConnectedPad pad)
            {
                if (!Pool.Contains(pad)) Pool.Add(pad);
                return Pool.IndexOf(pad);
            }

            public SimulatedPad Add()
            {
                SimulatedPad pad = new SimulatedPad();
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
                text.Append("pads " + string.Join(" ", m.Pads.Select(p => $"{Index(p)}:{p.Id},{p.Name},{p.Guid},{p.Path ?? "null"},{p.Vendor:X4}:{p.Product:X4},{p.IsOpen},{p.Type},{m.Players.PlayerOf(p)}")) + "\n");
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
        public void A_manager_opens_seats_lights_announces_and_reads_its_pads_as_recorded()
        {
            var golden = Golden("Manager");
            var random = new Random(63);
            for (int run = 0; run < 150; run++)
            {
                bool start = random.Next(5) != 0;
                Side s = new(start);
                for (int i = random.Next(0, 4); i > 0; i--) { SimulatedPad pad = s.Add(); pad.Path = $"/dev/input/start{i}"; s.Set.Connect(pad); }
                if (random.Next(3) == 0) { s.Manager.Dispose(); s.Manager = s.Make(start); }
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
                            ushort product = random.Next(3) == 0 ? (ushort)0xE310 : (ushort)random.Next(0x10000);
                            SimulatedPad made = s.Add();
                            made.Name = name; made.Guid = guid; made.Path = path; made.Type = type; made.Vendor = 0x1A86; made.Product = product;
                            s.Set.Connect(made);
                            what = $"connect {name} {guid} {path}";
                            break;
                        case 1 when s.Twins.Count > 0:
                            int gone = random.Next(s.Twins.Count);
                            s.Set.Disconnect(s.Twins[gone]);
                            what = $"disconnect {gone}";
                            break;
                        case 2 when s.Twins.Count > 0:
                            int back = random.Next(s.Twins.Count);
                            s.Set.Connect(s.Twins[back]);
                            what = $"connect again {back}";
                            break;
                        case 3: case 4: case 5: s.Manager.Poll(); what = "poll"; break;
                        case 6: case 7: case 8: case 9:
                            if (s.Twins.Count == 0) break;
                            int touched = random.Next(s.Twins.Count);
                            int seed = random.Next();
                            what = $"touch {touched}: {Touch(new Random(seed), s.Twins[touched])}";
                            break;
                        case 10 when s.Pool.Count > 0:
                            int moved = random.Next(s.Pool.Count), seat = random.Next(-1, 10);
                            what = $"assign {moved} to {seat}: {Outcome(() => { s.Manager.Assign(s.Pool[moved], seat); return "ok"; })}";
                            break;
                        case 11: int forgot = random.Next(0, 10); s.Manager.Players.Forget(forgot); what = $"forget {forgot}"; break;
                        case 12:
                            int setting = random.Next(5);
                            double value = Readings[random.Next(Readings.Length)];
                            switch (setting)
                            {
                                case 0: s.Manager.FirstControllerOnly ^= true; break;
                                case 1: s.Manager.AnalogStickAsDpad ^= true; break;
                                case 2: s.Manager.LeftStickIsAnalog ^= true; break;
                                case 3: s.Manager.StickDeadzone = value; break;
                                default: s.Manager.AnalogDeadzone = value; break;
                            }
                            what = $"setting {setting} {value}";
                            break;
                        case 13:
                            PadButton rebound = Buttons[random.Next(Buttons.Length)];
                            SDL.GamepadButton to = SdlButtons[random.Next(SdlButtons.Length)];
                            bool unbind = random.Next(4) == 0, second = random.Next(3) == 0;
                            var map = new GamepadBindingMap();
                            if (unbind) map.Unbind(rebound); else map.Rebind(rebound, to);
                            if (second) s.Manager.PlayerBindings = p => p == 2 ? map : null;
                            else s.Manager.Bindings = map;
                            what = $"bind {rebound} {(unbind ? "nothing" : to.ToString())} {(second ? "player 2" : "all")}";
                            break;
                        case 14:
                            int[] kept = Enumerable.Range(0, s.Twins.Count).Where(_ => random.Next(2) == 0).ToArray();
                            s.Set = new SimulatedPads();
                            foreach (int k in kept) s.Set.Connect(s.Twins[k]);
                            s.Manager.UseDevices(s.Set);
                            what = $"use devices [{string.Join(",", kept)}]";
                            break;
                        case 15: s.Manager.Start(); what = "start"; break;
                        case 16 when random.Next(4) == 0: s.Manager.Dispose(); what = "dispose"; break;
                        case 17 when s.Twins.Count > 0 && random.Next(4) == 0:
                            int only = random.Next(s.Twins.Count);
                            s.Manager.Simulated = s.Twins[only];
                            s.Set = (SimulatedPads)s.Manager.Devices;
                            what = $"simulated {only}";
                            break;
                    }
                    golden.AddChunked("step", $"{what}\n{s.Dump()}", 64);
                }
                s.Manager.Dispose();
                golden.AddChunked("disposed", s.Dump(), 64);
            }
            golden.Check(100);
        }

        // The comparisons a random walk seldom lands on: a reading exactly at a deadzone, a stick one step either side of its threshold, and a deadzone outside its range.
        [Fact]
        public void A_reading_at_the_edge_of_a_deadzone_or_a_threshold_falls_the_recorded_side()
        {
            var golden = Golden("Edges");
            Side s = new(true);
            s.Set.Connect(s.Add());
            s.Manager.Poll();
            // The poll took the devices' word that a pad was plugged in, so there is none left to take.
            Assert.False(s.Set.DevicesChanged());
            int[] steps = { 0, 1, 2, 1637, 1638, 1639, 3276, 3277, 16383, 16384, 31128, 31129, 32766, 32767 };
            double[] deadzones = { 0, -1, 0.05, 0.1, 0.5, 0.95, 1, 2, double.NaN, 1638 / 32767.0, 16384 / 32767.0, 3277 / 32767.0, 1 / 32767.0 };
            foreach (double deadzone in deadzones)
                foreach (int magnitude in steps)
                    foreach (int sign in new[] { 1, -1 })
                    {
                        double value = sign * magnitude / 32767.0;
                        s.Manager.StickDeadzone = deadzone;
                        s.Manager.AnalogDeadzone = deadzone;
                        foreach (SDL.GamepadAxis axis in SdlAxes) s.Twins[0].SetAxis(axis, value);
                        golden.Add($"deadzone {Bits(deadzone)} reading {Bits(value)}",
                            $"{s.Twins[0].Axis(SDL.GamepadAxis.LeftX)} {string.Concat(Buttons.Select(b => s.Manager.IsPressed(b) ? '1' : '0'))} {string.Join(",", Axes.Select(a => Bits(s.Manager.Axis(a))))} {Bits(s.Manager.RawAxis(SDL.GamepadAxis.LeftY))}");
                    }
            golden.Check(300);
            s.Manager.Dispose();
        }

        private static bool RescanDue(long now, long last) => DeviceNative.PadsRescanDue(now, last) != 0;

        [Fact]
        public void A_rescan_is_due_at_the_recorded_ticks_and_first_at_once()
        {
            var golden = Golden("Rescan");
            var random = new Random(64);
            long[] edges = { 0, 1, -1, 9_999_999, 10_000_000, 10_000_001, -10_000_000, TimeSpan.FromDays(1).Ticks, long.MaxValue / 4, long.MinValue / 4 };
            for (int i = 0; i < 20_000; i++)
            {
                long now = random.Next(3) == 0 ? edges[random.Next(edges.Length)] : random.NextInt64(-50_000_000, 50_000_000);
                long last = random.Next(3) == 0 ? edges[random.Next(edges.Length)] : now - random.NextInt64(-20_000_000, 20_000_000);
                golden.AddChunked("due", $"{now} {last} {RescanDue(now, last)}", 1000);
            }
            golden.Check(20);
            Assert.Equal(TimeSpan.FromSeconds(-1), new GamepadManager(new GamepadBindingMap(), start: false, new SimulatedPads()).LastRescan);
            // A difference past what a tick count holds, for which the deleted C# threw, is a rescan.
            Assert.Equal(1, DeviceNative.PadsRescanDue(long.MaxValue, long.MinValue));
        }

        // SDL's virtual joysticks under the manager, which is shown only these, so no pad on the desk is opened.
        private static string OnVirtualPads()
        {
            var mine = new HashSet<uint>();
            var joysticks = new Dictionary<uint, IntPtr>();
            var names = new List<IntPtr>();
            var log = new List<string>();
            var seen = new List<string>();
            // A GUID and a path are SDL's own, so each is written as the order it was first seen in, which keeps which pads share one.
            string Plain(string? text, string kind)
            {
                if (text is null) return "null";
                if (!seen.Contains(kind + text)) seen.Add(kind + text);
                return $"{kind}{seen.IndexOf(kind + text)}";
            }
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
                using GamepadManager manager = new GamepadManager(new GamepadBindingMap(), start: true, new SdlPadDevices(), mine);
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
                        manager.Poll();
                        if (manager.Pads.Count == wanted) break;
                        System.Threading.Thread.Sleep(50);
                    }
                    foreach (ConnectedPad pad in manager.Pads)
                        if (!order.Contains(pad)) order.Add(pad);
                    // An id is SDL's own count of every joystick the process has attached, so a pad is told by its place and not its id.
                    log.Add("pads " + string.Join(" ", manager.Pads.Select(p => $"{order.IndexOf(p)}:{p.Name},{Plain(p.Guid, "guid")},{Plain(p.Path, "path")},{p.Vendor:X4}:{p.Product:X4},{p.Type},{p.IsOpen},{manager.Players.PlayerOf(p)},{p.ButtonLabel(SDL.GamepadButton.South) ?? "null"}")));
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

        // A layer the library has nothing behind: any IPadDevices but SDL's and the simulated set.
        public class Stranger : System.Reflection.DispatchProxy
        {
            protected override object? Invoke(System.Reflection.MethodInfo? method, object?[]? args) => throw new NotSupportedException(method?.Name);
        }

        [Fact]
        public void A_device_layer_the_library_cannot_read_is_refused_in_words_at_the_start_and_on_a_swap()
        {
            IPadDevices stranger = System.Reflection.DispatchProxy.Create<IPadDevices, Stranger>();
            var refused = Assert.Throws<ArgumentException>(() => new GamepadManager(new GamepadBindingMap(), false, stranger));
            Assert.Contains("is not a device layer the platform library reads; GamepadManager takes SdlPadDevices or SimulatedPads.", refused.Message);
            var set = new SimulatedPads();
            using var manager = new GamepadManager(new GamepadBindingMap(), false, set);
            Assert.Throws<ArgumentException>(() => manager.UseDevices(stranger));
            Assert.Same(set, manager.Devices);
        }

        [Fact]
        public void Sdls_virtual_pads_are_opened_read_and_replaced_as_recorded_through_the_librarys_sdl_layer()
        {
            var golden = Golden("VirtualPads");
            string log = OnVirtualPads();
            Assert.Contains("changed", log);
            foreach (var (line, i) in log.Split('\n').Select((l, i) => (l, i))) golden.Add($"line {i}", line);
            golden.Check(10);
        }

        // What of a player does not depend on how fast the dummy device drains.
        private static string Steady(AudioPlayer player) =>
            $"available {player.IsAvailable} rate {player.SampleRate} target {player.RateControl.TargetQueuedFrames} deviation {Bits(player.RateControl.MaxDeviation)} volume {BitConverter.SingleToInt32Bits(player.Volume):X8} " +
            $"in {player.RateControl.TotalInputFrames} shed {player.RateControl.SheddingEvents} {player.RateControl.IsShedding}";

        [Fact]
        public void The_games_audio_player_opens_reopens_and_steers_the_dummy_device_as_recorded()
        {
            SDL.SetHint(SDL.Hints.AudioDriver, "dummy");
            var golden = Golden("Audio");
            // Targets no queue here comes near, and one below zero, which sheds from the first sample: nothing recorded then turns on how fast the dummy device drains.
            foreach ((int rate, int latency, double deviation, int frames) in new[] { (32000, 256, 0.005, 4096), (44100, 100, 0.02, 1024), (48000, 5000, 0.0, 512), (96000, -5, 0.005, 2048), (48000, 100_000, 0.01, 4096) })
            {
                var player = new AudioPlayer(rate, latency, deviation, frames);
                Assert.Equal("dummy", SDL.GetCurrentAudioDriver());
                void Do(string what, Action<AudioPlayer> act)
                {
                    act(player);
                    golden.Add($"{rate} {latency} after {what}", Steady(player));
                }
                Do("opening", _ => { });
                foreach (float volume in new[] { 0.5f, -1f, 2f, float.NaN, 0f, 1f, 0.25f }) Do($"volume {volume}", p => p.Volume = volume);
                Do("null", p => p.Submit(null!, rate));
                Do("empty", p => p.Submit(Array.Empty<short>(), rate));
                short[] tone = Enumerable.Range(0, 2 * 800).Select(i => (short)(Math.Sin(i * 0.05) * 8000)).ToArray();
                Do("a tone", p => p.Submit(tone, rate));
                // A new rate with nothing to play reopens the device and resets the rate control, and steers nothing after.
                Do("empty at a new rate", p => p.Submit(Array.Empty<short>(), rate + 1000));
                golden.Add($"{rate} {latency} last after the reopening", Bits(player.RateControl.LastRatio));
                Assert.Equal(Bits(player.RateControl.NominalRatio), Bits(player.RateControl.LastRatio));
                Do("one sample", p => p.Submit(new short[] { 7 }, rate + 1000));
                Do("a rate of zero", p => p.Submit(tone, 0));
                Do("a rate below zero", p => p.Submit(tone, -44100));
                Do("back to the first rate", p => p.Submit(tone, rate));
                Assert.InRange(player.QueuedFrames, 0, 4000);
                // The output count follows the queue, which drains in real time: within the deviation of the four tones' 3,200 frames, or nothing while shedding.
                if (player.IsAvailable)
                    Assert.InRange(player.RateControl.TotalOutputFrames, latency > 0 ? (long)(3190 * (1 - deviation)) - 8 : 0, latency > 0 ? (long)(3200 * (1 + deviation)) + 8 : 0);
                Do("disposing", p => p.Dispose());
                Assert.Equal((false, 0, 0), (player.IsAvailable, player.SampleRate, player.QueuedFrames));
                Do("a tone after disposing", p => p.Submit(tone, rate));
                Do("volume after disposing", p => p.Volume = 0.75f);
            }
            golden.Check(50);
        }

        private static byte[]? Decode(string path) => UiSoundPlayer.Decode(path);

        [Fact]
        public void The_interface_sounds_decode_to_the_recorded_bytes_and_one_replaces_another_as_recorded()
        {
            SDL.SetHint(SDL.Hints.AudioDriver, "dummy");
            var golden = Golden("UiSounds");
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
                for (int i = 0; i < files.Count; i++)
                    golden.Add($"decode {i}", Decode(files[i]) is { } bytes ? $"{bytes.Length} {Convert.ToHexString(SHA256.HashData(bytes))[..16]}" : "null");
                Assert.True(Decode(files[3])!.Length > 64 * 1024, "the long sound is past the first buffer, so it was taken");

                var player = new UiSoundPlayer();
                int seen = 0;
                void Do(Action<UiSoundPlayer> act)
                {
                    act(player);
                    golden.Add($"seen {seen++}", $"open {player.IsOpen} volume {BitConverter.SingleToInt32Bits(player.Volume):X8}");
                }
                Do(_ => { });
                // Silent from here, so a run plays nothing if SDL should open a device that is not the dummy one.
                foreach (float volume in new[] { 0.5f, 3f, -3f, float.NaN, 0f }) Do(p => p.Volume = volume);
                Do(p => p.Play(files[6]));
                Do(p => p.Play(files[5]));
                Do(p => p.Play(files[4]));
                Assert.False(player.IsOpen, "a sound that is missing, unreadable or empty opens nothing");
                Do(p => p.Preload(files));
                int whole = Decode(files[3])!.Length, shorter = Decode(files[1])!.Length;
                Do(p => p.Play(files[3]));
                if (player.IsOpen)
                {
                    Assert.InRange(player.Queued, whole / 2, whole);
                    Do(p => p.Play(files[1]));
                    Assert.InRange(player.Queued, shorter / 2, shorter);
                    byte[] remembered = new byte[48000 * 8];
                    Do(p => p.Remember("a name that is no file", remembered));
                    Do(p => p.Play("a name that is no file"));
                    Assert.InRange(player.Queued, remembered.Length / 2, remembered.Length);
                    Do(p => p.Remember("nothing", Array.Empty<byte>()));
                    Do(p => p.Play("nothing"));
                    Assert.InRange(player.Queued, remembered.Length / 4, remembered.Length);
                }
                Do(p => p.Dispose());
                Assert.Equal((false, 0), (player.IsOpen, player.Queued));
                Do(p => p.Play(files[1]));
                Assert.False(player.IsOpen, "a player disposed does not open again");
                golden.Check(20);
            }
            finally
            {
                if (Directory.Exists(folder)) Directory.Delete(folder, recursive: true);
            }
        }

        [Fact]
        public void The_library_has_every_export_the_device_half_calls()
        {
            string header = File.ReadAllText(Path.Combine(EmuSen.Galaxia.ConfigRoot.Directory, "Platform", "include", "emusen_platform.h"));
            foreach (string export in DeviceNative.Exports) Assert.Contains(export + "(", header);
        }
    }
}
