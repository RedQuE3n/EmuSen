using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using EmuSen.Endymion.Input;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Input
{
    // Runs only with EMUSEN_UINPUT_PADS=1 where /dev/uinput is writable, since the pads it makes are seen by every program on the desk.
    public sealed class UinputTheoryAttribute : TheoryAttribute
    {
        public UinputTheoryAttribute() => Skip = UinputFactAttribute.Why();
    }

    public sealed class UinputFactAttribute : FactAttribute
    {
        public UinputFactAttribute() => Skip = Why();

        internal static string? Why() =>
            Environment.GetEnvironmentVariable("EMUSEN_UINPUT_PADS") != "1"
                ? "Makes kernel pads through /dev/uinput; set EMUSEN_UINPUT_PADS=1 - see EmuSen_Input.md §8.12"
                : UinputPad.Unavailable;
    }

    // Real kernel pads plugged, pulled and plugged back under the real SDL layer, polled as Mistress polls, under each manager - see EmuSen_Input.md §8.12.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class UinputHotplugTests(ITestOutputHelper output)
    {
        [DllImport("libc")] private static extern int setenv(string name, string value, int overwrite);

        // The event nodes of the pads this test made; SDL names a pad from its own database, so a pad is told by its path.
        private static readonly HashSet<string> Paths = new();

        private static bool Mine(uint id) => SDL.GetGamepadPathForID(id) is { } path && Paths.Contains(path);

        // Only the pads this test made, so a pad on the desk takes no seat.
        private sealed class MineOnly(SdlPadDevices sdl) : IPadDevices
        {
            public bool Init() => sdl.Init();
            public void Quit() => sdl.Quit();
            public uint[] Attached() => sdl.Attached().Where(Mine).ToArray();
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
            public uint VendorProduct(IntPtr pad) => sdl.VendorProduct(pad);
            public void SetPlayerIndex(IntPtr pad, int index) => sdl.SetPlayerIndex(pad, index);
        }

        // A manager on the real devices, ticked as Mistress's 16 ms pad timer ticks it: Poll and nothing else on the thread.
        private sealed class Rig : IDisposable
        {
            private readonly ITestOutputHelper _output;
            private readonly Stopwatch _clock = Stopwatch.StartNew();
            private readonly HashSet<uint> _only = new();
            private readonly List<UinputPad> _made = new();
            public readonly GamepadManager Manager;
            public readonly List<PadConnection> Changes = new();

            public Rig(bool native, ITestOutputHelper output, params UinputPad[] present)
            {
                _output = output;
                _made.AddRange(present);
                Manager = native
                    ? new GamepadManager(new GamepadBindingMap(), start: false, new SdlPadDevices(), true, _only)
                    : new GamepadManager(new GamepadBindingMap(), start: false, new MineOnly(new SdlPadDevices()), false);
                Manager.PadChanged += change =>
                {
                    Changes.Add(change);
                    Say($"{(change.Connected ? "connected" : "disconnected")}: {change.Pad.Name} player {change.Player} path {change.Pad.Path} guid {change.Pad.Guid}");
                };
                Manager.Start();
                Say($"native {native}, started");
            }

            public void Say(string line) => _output.WriteLine($"{_clock.Elapsed.TotalSeconds,7:F3} {line}");

            public UinputPad Make(string name, ushort vendor = UinputPad.XboxVendor, ushort product = UinputPad.Xbox360Product, bool mine = true)
            {
                var pad = new UinputPad(name, vendor, product);
                _made.Add(pad);
                if (mine && pad.EventPath is { } path) Paths.Add(path);
                Say($"made '{name}' {vendor:x4}:{product:x4} at {pad.EventPath}{(mine ? "" : ", not seated")}");
                return pad;
            }

            public void Pull(UinputPad pad)
            {
                pad.Dispose();
                if (pad.EventPath is { } path) Paths.Remove(path);
                Say($"pulled '{pad.Name}'");
            }

            public void Tick()
            {
                // The library is told the ids the test may see, as the C# is told by MineOnly.
                if (SDL.GetGamepads(out int count) is { } ids)
                    foreach (uint id in ids.Take(count).Where(Mine)) _only.Add(id);
                Manager.Poll();
                Thread.Sleep(16);
            }

            public bool TickUntil(Func<bool> condition, double seconds = 4)
            {
                var until = Stopwatch.StartNew();
                while (until.Elapsed.TotalSeconds < seconds)
                {
                    Tick();
                    if (condition()) return true;
                }
                return false;
            }

            public int Open => Manager.Pads.Count;

            // Waits for the count of open pads and says what the manager holds.
            public void Expect(int open, string what)
            {
                bool held = TickUntil(() => Open == open);
                Say($"{what}: {State()}");
                Assert.True(held, $"{what}: {open} open pads expected, {State()}");
            }

            public string State() =>
                "seats " + string.Join(" ", Enumerable.Range(1, 4).Select(p => Manager.Players.SeatOf(p) is { } s ? $"{p}={s.Path}{(s.IsOpen ? "" : "(gone)")}" : $"{p}=-"));

            public int PlayerAt(UinputPad pad) => Manager.Pads.FirstOrDefault(p => p.Path == pad.EventPath) is { } open ? Manager.Players.PlayerOf(open) : 0;

            // The pad's South is heard as the player's B and as no other player's.
            public void Heard(UinputPad pad, int player)
            {
                pad.Press(UinputPad.South, true);
                bool heard = TickUntil(() => Manager.IsPressed(PadButton.B, player), 1);
                bool others = Enumerable.Range(1, 4).Any(p => p != player && Manager.IsPressed(PadButton.B, p));
                pad.Press(UinputPad.South, false);
                TickUntil(() => !Manager.IsPressed(PadButton.B, player), 1);
                Assert.True(heard && !others, $"'{pad.Name}' heard as player {player} alone: {heard}, others {others}");
            }

            public void Dispose()
            {
                Manager.Dispose();
                foreach (UinputPad pad in _made) pad.Dispose();
            }
        }

        private const string Pad = "EmuSen uinput pad";

        // A pad made before the manager starts, as one plugged in before Mistress is.
        private static UinputPad Present(string name)
        {
            var pad = new UinputPad(name);
            if (pad.EventPath is { } path) Paths.Add(path);
            return pad;
        }

        // Present at start, pulled, and plugged back as the same pad on the same node: player 1 again, and heard.
        [UinputTheory, InlineData(false), InlineData(true)]
        public void A_pad_pulled_and_plugged_back_is_player_1_again(bool native)
        {
            UinputPad pad = Present(Pad);
            using var rig = new Rig(Native(native), output, pad);
            rig.Expect(1, "present at start");
            rig.Heard(pad, 1);
            rig.Pull(pad);
            rig.Expect(0, "pulled");
            pad = rig.Make(Pad);
            rig.Expect(1, "plugged back");
            Assert.Equal(1, rig.PlayerAt(pad));
            rig.Heard(pad, 1);
        }

        // None at start, then one plugged in for the first time: seen, player 1, and heard.
        [UinputTheory, InlineData(false), InlineData(true)]
        public void A_pad_plugged_in_after_start_is_seen(bool native)
        {
            using var rig = new Rig(Native(native), output);
            rig.Expect(0, "started with none");
            UinputPad pad = rig.Make(Pad);
            rig.Expect(1, "plugged in");
            Assert.Equal((true, 1), (rig.Changes[^1].Connected, rig.Changes[^1].Player));
            rig.Heard(pad, 1);
        }

        // Plugged back while another device holds its old node, as a Bluetooth reconnect or a different port gives it: the same GUID on a new path finds the seat.
        [UinputTheory, InlineData(false), InlineData(true)]
        public void A_pad_plugged_back_on_another_node_is_its_player_again(bool native)
        {
            using var rig = new Rig(Native(native), output);
            UinputPad pad = rig.Make(Pad);
            rig.Expect(1, "present");
            string? before = pad.EventPath;
            rig.Pull(pad);
            rig.Expect(0, "pulled");
            rig.Make("EmuSen placeholder", mine: false);
            pad = rig.Make(Pad);
            rig.Expect(1, "plugged back");
            Assert.NotEqual(before, pad.EventPath);
            Assert.Equal(1, rig.PlayerAt(pad));
            rig.Heard(pad, 1);
        }

        // Two pads of one model; player 1's pulled and plugged back elsewhere takes player 1, and player 2's stays player 2's throughout.
        [UinputTheory, InlineData(false), InlineData(true)]
        public void Of_two_pads_the_one_replugged_gets_its_player_back_and_the_other_keeps_its_own(bool native)
        {
            using var rig = new Rig(Native(native), output);
            UinputPad first = rig.Make(Pad);
            rig.Expect(1, "first");
            UinputPad second = rig.Make(Pad);
            rig.Expect(2, "second");
            Assert.Equal((1, 2), (rig.PlayerAt(first), rig.PlayerAt(second)));
            rig.Pull(first);
            rig.Expect(1, "player 1's pulled");
            rig.Heard(second, 2);
            rig.Make("EmuSen placeholder", mine: false);
            first = rig.Make(Pad);
            rig.Expect(2, "player 1's plugged back");
            Assert.Equal((1, 2), (rig.PlayerAt(first), rig.PlayerAt(second)));
            rig.Heard(first, 1);
            rig.Heard(second, 2);
        }

        // Steam Input's virtual pads in Game Mode: a controller's reconnect removes them and makes them again on new nodes; each slot's pad finds its own seat, in whichever order they return.
        [UinputTheory, InlineData(false), InlineData(true)]
        public void Steams_virtual_pads_made_again_after_a_reconnect_find_their_own_players(bool native)
        {
            AllowSteamVirtualPads();
            using var rig = new Rig(Native(native), output);
            UinputPad builtIn = rig.Make("Microsoft X-Box 360 pad 0", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
            rig.Expect(1, "slot 0");
            UinputPad xbox = rig.Make("Microsoft X-Box 360 pad 1", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
            rig.Expect(2, "slot 1");
            Assert.Equal((1, 2), (rig.PlayerAt(builtIn), rig.PlayerAt(xbox)));
            Assert.NotEqual(rig.Manager.Players.SeatOf(1)!.Guid, rig.Manager.Players.SeatOf(2)!.Guid);

            rig.Pull(xbox);
            rig.Pull(builtIn);
            rig.Expect(0, "both removed");
            rig.Make("EmuSen placeholder", mine: false);
            xbox = rig.Make("Microsoft X-Box 360 pad 1", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
            rig.Expect(1, "slot 1 made again first");
            builtIn = rig.Make("Microsoft X-Box 360 pad 0", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
            rig.Expect(2, "slot 0 made again");
            Assert.Equal((1, 2), (rig.PlayerAt(builtIn), rig.PlayerAt(xbox)));
            rig.Heard(xbox, 2);
            rig.Heard(builtIn, 1);
        }

        // A reconnect Steam gives another slot is another GUID; beside a pad still connected it takes the lowest seat free, which is its own.
        [UinputTheory, InlineData(false), InlineData(true)]
        public void A_steam_pad_back_in_another_slot_takes_the_seat_left_free(bool native)
        {
            AllowSteamVirtualPads();
            using var rig = new Rig(Native(native), output);
            UinputPad builtIn = rig.Make("Microsoft X-Box 360 pad 0", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
            rig.Expect(1, "slot 0");
            UinputPad xbox = rig.Make("Microsoft X-Box 360 pad 1", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
            rig.Expect(2, "slot 1");
            rig.Pull(xbox);
            rig.Expect(1, "slot 1 removed");
            xbox = rig.Make("Microsoft X-Box 360 pad 2", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
            rig.Expect(2, "slot 2 made");
            Assert.Equal((1, 2), (rig.PlayerAt(builtIn), rig.PlayerAt(xbox)));
            rig.Heard(xbox, 2);
        }

        // Steam's file names the controller behind each virtual pad; SDL reports its vendor and product, so the built-in controls are known and the external pad is player 1 - see EmuSen_Input.md §8.13.2.
        [UinputTheory, InlineData(false), InlineData(true)]
        public void Steams_virtual_pads_report_the_controllers_behind_them_and_the_external_one_is_player_1(bool native)
        {
            AllowSteamVirtualPads();
            // SDL reads the file at most every three seconds and skips one whose time it read last, neither forgotten when its joysticks stop.
            if (SinceSteamInfo.IsRunning && SinceSteamInfo.Elapsed < TimeSpan.FromSeconds(3.2)) Thread.Sleep(TimeSpan.FromSeconds(3.2) - SinceSteamInfo.Elapsed);
            string info = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"EmuSenSteamVirtualGamepadInfo{Guid.NewGuid():N}.txt");
            // Steam's own format, as steamclient.so writes it: "[slot %d]", "name=%s", "VID=0x%.4x", "PID=0x%.4x", "type=%s".
            System.IO.File.WriteAllText(info, "[slot 0]\nname=Legion Go S Controller\nVID=0x28de\nPID=0x12ff\ntype=xbox360\n[slot 1]\nname=Xbox Wireless Controller\nVID=0x045e\nPID=0x0b13\ntype=xboxone\n");
            System.IO.File.SetLastWriteTimeUtc(info, DateTime.UtcNow.AddSeconds(-10 * Interlocked.Increment(ref _steamInfoFiles)));
            setenv("SteamVirtualGamepadInfo", info, 1);
            try
            {
                using var rig = new Rig(Native(native), output);
                rig.Manager.ExternalControllerFirst = true;
                UinputPad builtIn = rig.Make("Microsoft X-Box 360 pad 0", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
                rig.Expect(1, "slot 0");
                UinputPad xbox = rig.Make("Microsoft X-Box 360 pad 1", UinputPad.SteamVendor, UinputPad.SteamVirtualProduct);
                rig.Expect(2, "slot 1");
                ConnectedPad Open(UinputPad pad) => rig.Manager.Pads.Single(p => p.Path == pad.EventPath);
                foreach (UinputPad pad in new[] { builtIn, xbox })
                    rig.Say($"'{pad.Name}': name '{Open(pad).Name}', vendor {Open(pad).Vendor:x4}, product {Open(pad).Product:x4}, type {Open(pad).Type}, guid {Open(pad).Guid}");
                Assert.Equal((0x28DE, 0x12FF, true), (Open(builtIn).Vendor, Open(builtIn).Product, Open(builtIn).IsBuiltIn));
                Assert.Equal((0x045E, 0x0B13, false), (Open(xbox).Vendor, Open(xbox).Product, Open(xbox).IsBuiltIn));
                Assert.Equal(("Legion Go S Controller", "Xbox Wireless Controller"), (Open(builtIn).Name, Open(xbox).Name));
                Assert.Equal((2, 1), (rig.PlayerAt(builtIn), rig.PlayerAt(xbox)));
                rig.Heard(xbox, 1);
                rig.Heard(builtIn, 2);
            }
            finally
            {
                setenv("SteamVirtualGamepadInfo", "", 1);
                System.IO.File.Delete(info);
                SinceSteamInfo.Restart();
            }
        }

        // The pump and the flush on the poll's path allocate nothing, with a pad open and pressed between the calls.
        [UinputFact]
        public void The_pump_and_the_flush_allocate_nothing()
        {
            var pad = new UinputPad(Pad);
            var sdl = new SdlPadDevices();
            Assert.True(sdl.Init(), SDL.GetError());
            try
            {
                Thread.Sleep(200);
                sdl.Update();
                IntPtr open = sdl.Open(sdl.Attached()[0]);
                for (int i = 0; i < 100; i++) { sdl.DevicesChanged(); sdl.Update(); }
                long before = GC.GetAllocatedBytesForCurrentThread();
                for (int i = 0; i < 1000; i++)
                {
                    pad.Press(UinputPad.South, (i & 1) == 0);
                    sdl.DevicesChanged();
                    sdl.Update();
                }
                long allocated = GC.GetAllocatedBytesForCurrentThread() - before;
                bool held = sdl.Button(open, SDL.GamepadButton.South);
                sdl.Close(open);
                Assert.Equal(0, allocated);
                Assert.False(held);
            }
            finally
            {
                pad.Dispose();
                sdl.Quit();
            }
        }

        private static readonly Stopwatch SinceSteamInfo = new();
        private static int _steamInfoFiles;

        // SDL hides Steam's virtual pads unless told it runs under Steam, which Steam tells it through this variable; SDL reads the C environment.
        private static void AllowSteamVirtualPads() => setenv("SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD", "1", 1);

        private static bool Native(bool native)
        {
            if (native) Assert.True(DeviceNative.Active, DeviceNative.Report);
            return native;
        }
    }
}
