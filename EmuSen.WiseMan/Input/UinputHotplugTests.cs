using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading;
using EmuSen.Endymion.Input;
using EmuSen.Endymion.Native;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Input
{
    // Runs only with EMUSEN_UINPUT_PADS=1 where /dev/uinput is writable, since the pads it makes are seen by every program on the desk.
    public sealed class UinputFactAttribute : FactAttribute
    {
        public UinputFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_UINPUT_PADS") != "1") Skip = "Makes kernel pads through /dev/uinput; set EMUSEN_UINPUT_PADS=1 - see EmuSen_Input.md §8.12";
            else if (UinputPad.Unavailable is { } why) Skip = why;
        }
    }

    // Real kernel pads plugged, pulled and plugged back under the real SDL layer, polled as Mistress polls - see EmuSen_Input.md §8.12.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class UinputHotplugTests(ITestOutputHelper output)
    {
        private const string Prefix = "EmuSen uinput pad";

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
            public void SetPlayerIndex(IntPtr pad, int index) => sdl.SetPlayerIndex(pad, index);
        }

        private sealed class Rig : IDisposable
        {
            private readonly ITestOutputHelper _output;
            private readonly Stopwatch _clock = Stopwatch.StartNew();
            private readonly HashSet<uint> _only = new();
            public readonly GamepadManager Manager;
            public readonly List<PadConnection> Changes = new();

            public Rig(bool native, ITestOutputHelper output)
            {
                _output = output;
                Manager = native
                    ? new GamepadManager(new GamepadBindingMap(), start: false, new SdlPadDevices(), true, _only)
                    : new GamepadManager(new GamepadBindingMap(), start: false, new MineOnly(new SdlPadDevices()), false);
                Manager.PadChanged += change =>
                {
                    Changes.Add(change);
                    Say($"changed {change.Pad.Name} {(change.Connected ? "in" : "out")} player {change.Player} id {change.Pad.Id} path {change.Pad.Path}");
                };
                Manager.Start();
            }

            public void Say(string line) => _output.WriteLine($"{_clock.Elapsed.TotalSeconds,8:F3} {line}");

            // One 16 ms tick of Mistress's pad timer: Poll and nothing else on the thread.
            public void Tick()
            {
                foreach (uint id in SdlIds()) _only.Add(id);
                Manager.Poll();
                Thread.Sleep(16);
            }

            public static uint[] SdlIds() => SDL.GetGamepads(out int count) is { } ids ? ids.Take(count).Where(Mine).ToArray() : [];

            // Ticks until the condition holds or the time runs out; true when it held.
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

            public string State() =>
                $"sdl [{string.Join(",", SdlIds())}] pads [{string.Join(", ", Manager.Pads.Select(p => $"{p.Id}:{p.Name}:p{Manager.Players.PlayerOf(p)}:{(p.IsOpen ? "open" : "closed")}"))}]";

            public bool Holds(int player, SDL.GamepadButton button) => Manager.Players.PadFor(player)?.IsRawPressed(button) == true;

            public void Dispose() => Manager.Dispose();
        }

        private static UinputPad Make(string name)
        {
            var pad = new UinputPad(name);
            if (pad.EventPath is { } path) Paths.Add(path);
            return pad;
        }

        private static string Unique(string what) => $"{Prefix} {what} {Environment.ProcessId}";

        // Present at start, pressed, pulled and plugged back as the same pad on the same node: player 1 again, and heard.
        private void Replug(bool native)
        {
            var pad = Make(Unique("one"));
            using var rig = new Rig(native, output);
            try
            {
                rig.Say($"native {native}, made {pad.SysName} {pad.EventPath} before start: {rig.State()}");
                Assert.True(rig.TickUntil(() => rig.Manager.Pads.Count == 1), "the pad present at start");
                pad.Press(UinputPad.South, true);
                Assert.True(rig.TickUntil(() => rig.Holds(1, SDL.GamepadButton.South), 1), "its press");
                pad.Dispose();
                bool gone = rig.TickUntil(() => rig.Manager.Pads.Count == 0);
                rig.Say($"pulled, seen {gone}: {rig.State()}");
                Assert.True(gone, "the pull");

                pad = Make(Unique("one"));
                bool back = rig.TickUntil(() => rig.Manager.Pads.Count == 1);
                rig.Say($"plugged back {pad.SysName} {pad.EventPath}, seen {back}: {rig.State()}");
                Assert.True(back, "the pad plugged back");
                Assert.Equal((true, 1), (rig.Changes[^1].Connected, rig.Changes[^1].Player));
                pad.Press(UinputPad.South, true);
                Assert.True(rig.TickUntil(() => rig.Holds(1, SDL.GamepadButton.South), 1), "its press after the replug");
            }
            finally
            {
                pad.Dispose();
            }
        }

        // None at start, then one plugged in: seen, seated as player 1 and heard.
        private void FirstPlug(bool native)
        {
            using var rig = new Rig(native, output);
            var pad = Make(Unique("late"));
            try
            {
                bool seen = rig.TickUntil(() => rig.Manager.Pads.Count == 1);
                rig.Say($"native {native}, plugged {pad.SysName} {pad.EventPath} after start, seen {seen}: {rig.State()}");
                Assert.True(seen, "a pad plugged in after start");
                Assert.Equal((true, 1), (rig.Changes[^1].Connected, rig.Changes[^1].Player));
                pad.Press(UinputPad.Start, true);
                Assert.True(rig.TickUntil(() => rig.Holds(1, SDL.GamepadButton.Start), 1), "its press");
            }
            finally
            {
                pad.Dispose();
            }
        }

        [UinputFact]
        public void A_kernel_pad_pulled_and_plugged_back_is_player_1_again_managed() => Replug(native: false);

        [UinputFact]
        public void A_kernel_pad_pulled_and_plugged_back_is_player_1_again_native() => Replug(native: Native());

        [UinputFact]
        public void A_kernel_pad_plugged_in_after_start_is_seen_managed() => FirstPlug(native: false);

        [UinputFact]
        public void A_kernel_pad_plugged_in_after_start_is_seen_native() => FirstPlug(native: Native());

        private static bool Native()
        {
            Assert.True(DeviceNative.Active, DeviceNative.Report);
            return true;
        }
    }
}
