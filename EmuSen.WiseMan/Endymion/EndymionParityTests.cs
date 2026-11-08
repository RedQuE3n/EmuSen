using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Crystal;
using SDL3;

namespace EmuSen.WiseMan.Endymion
{
    // Endymion's rules held to the answers its library and its C# both gave before the C# was deleted, and the pads' rules to Galaxia's C# that the frontends keep - see EmuSen_Backend.md §40.3.
    public class EndymionParityTests
    {
        public EndymionParityTests() => Assert.True(EndymionNative.Ready, EndymionNative.Report);

        private static ParityGolden Golden(string name) => new("EndymionParity." + name);

        private static string Bits(double value) => BitConverter.DoubleToInt64Bits(value).ToString("X16");

        private static void AssertNone(List<string> different, string what) =>
            Assert.True(different.Count == 0, $"{different.Count} {what} differ:\n{string.Join("\n", different.Take(20))}");

        // What a call came to: its answer, or the exception it threw with its words.
        private static string Outcome(Func<string> call)
        {
            try { return call(); }
            catch (Exception e) { return $"{e.GetType().Name}: {e.Message}"; }
        }

        // A stream of samples as a core makes them: a tone, noise, silence, the extremes and steps between them.
        private static short[] Stream(Random random, int frames)
        {
            var samples = new short[frames * 2 + (random.Next(5) == 0 ? 1 : 0)];
            int kind = random.Next(5);
            for (int i = 0; i < samples.Length; i++)
                samples[i] = kind switch
                {
                    0 => (short)(Math.Sin(i * 0.05) * 30000),
                    1 => (short)random.Next(short.MinValue, short.MaxValue + 1),
                    2 => 0,
                    3 => i % 2 == 0 ? short.MaxValue : short.MinValue,
                    _ => (short)(random.Next(3) - 1),
                };
            return samples;
        }

        private static readonly double[] Ratios = { 1.0, 0.995, 1.005, 0.5, 2.0, 1.0 / 3, 3.0, 0.9999999, 1.0000001, 44100.0 / 32000, 32000.0 / 48000, 1.5, 0.75, double.Epsilon * 1e300, 7.25 };

        [Fact]
        public void The_resampler_makes_the_recorded_samples_from_every_stream_at_every_ratio()
        {
            var golden = Golden("Resampler");
            var random = new Random(32000);
            for (int run = 0; run < 400; run++)
            {
                var resampler = new LinearResampler();
                for (int call = 0; call < 30; call++)
                {
                    short[] input = Stream(random, random.Next(0, 300));
                    double ratio = random.Next(3) == 0 ? Ratios[random.Next(Ratios.Length)] : 1 + (random.NextDouble() - 0.5) * 0.02;
                    bool reset = random.Next(25) == 0;
                    if (reset) resampler.Reset();
                    golden.AddChunked("resample", $"{reset} {Bits(ratio)} {input.Length}: {Outcome(() => string.Join(",", resampler.Resample(input, ratio)))}", 64);
                }
            }
            golden.Check(100);
        }

        [Fact]
        public void A_ratio_and_an_input_are_refused_as_recorded_and_a_ratio_that_is_not_finite_in_the_librarys_words()
        {
            var golden = Golden("Refusals");
            foreach (double ratio in new[] { 0.0, -0.0, -1.0, double.NegativeInfinity, double.MinValue })
                foreach (short[]? input in new short[]?[] { new short[] { 1, 2, 3, 4 }, Array.Empty<short>(), null })
                {
                    string answer = Outcome(() => string.Join(",", new LinearResampler().Resample(input!, ratio)));
                    golden.Add($"{Bits(ratio)} {input?.Length.ToString() ?? "null"}", answer);
                    Assert.StartsWith("ArgumentOutOfRangeException: Resample ratio must be positive.", answer);
                }
            golden.Add("null at 1", Outcome(() => string.Join(",", new LinearResampler().Resample(null!, 1.0))));
            golden.Check(16);
            Assert.Empty(new LinearResampler().Resample(Array.Empty<short>(), 1.0));
            Assert.Empty(new LinearResampler().Resample(new short[] { 5 }, 1.0));
            foreach (double ratio in new[] { double.NaN, double.PositiveInfinity })
                Assert.Equal("ArgumentOutOfRangeException: Resample ratio must be finite. (Parameter 'ratio')", Outcome(() => string.Join(",", new LinearResampler().Resample(new short[] { 1, 2 }, ratio))));
        }

        private static string State(DynamicRateControl control) =>
            $"{control.TargetQueuedFrames} {Bits(control.MaxDeviation)} {Bits(control.SheddingEntryFactor)} {Bits(control.SheddingExitFactor)} {Bits(control.NominalRatio)} {Bits(control.LastRatio)} {control.SheddingEvents} {control.IsShedding} {control.TotalInputFrames} {control.TotalOutputFrames}";

        [Fact]
        public void The_rate_control_steers_sheds_and_counts_as_recorded_through_every_queue()
        {
            var golden = Golden("RateControl");
            var random = new Random(48000);
            int[] targets = { 0, 1, 64, 1024, 8192, -5, 32000 };
            for (int run = 0; run < 300; run++)
            {
                int target = targets[random.Next(targets.Length)];
                var control = new DynamicRateControl(target);
                int queued = Math.Max(0, target);
                for (int step = 0; step < 60; step++)
                {
                    string what;
                    switch (random.Next(14))
                    {
                        case 0: control.MaxDeviation = random.Next(2) == 0 ? 0.005 : random.NextDouble() * 0.2; what = "deviation"; break;
                        case 1: control.SheddingEntryFactor = 1 + random.NextDouble() * 4; what = "entry"; break;
                        case 2: control.SheddingExitFactor = random.NextDouble() * 3; what = "exit"; break;
                        case 3: control.NominalRatio = random.Next(2) == 0 ? 1.0 : 0.9 + random.NextDouble() * 0.2; what = "nominal"; break;
                        case 4: control.TargetQueuedFrames = targets[random.Next(targets.Length)]; what = "target"; break;
                        case 5: control.Reset(); what = "reset"; break;
                        default:
                            queued = random.Next(4) == 0 ? random.Next(-100, Math.Max(1, target) * 5) : Math.Max(0, queued + random.Next(-300, 300));
                            short[] input = Stream(random, random.Next(0, 600));
                            string output = Outcome(() => string.Join(",", control.Process(input, queued)));
                            what = $"process {queued}: {output} ratio {Bits(control.ComputeRatio(queued))}";
                            break;
                    }
                    golden.AddChunked("step", $"{what} {State(control)}", 64);
                }
            }

            var edge = new DynamicRateControl(100);
            golden.Add("null", Outcome(() => string.Join(",", edge.Process(null!, 250))) + " " + State(edge));
            edge.NominalRatio = -1;
            golden.Add("nominal -1", Outcome(() => string.Join(",", edge.Process(new short[] { 1, 2, 3, 4 }, 100))) + " " + State(edge));
            golden.Check(100);
        }

        private static unsafe double ResolveNative(PadAxis axis, double analog, uint held)
        {
            double value;
            Assert.Equal(0, EndymionNative.PadResolve((uint)axis, analog, held, &value));
            return value;
        }

        private static unsafe double CombineNative(double analog, bool negative, bool positive)
        {
            double value;
            EndymionNative.PadCombine(analog, negative ? 1u : 0u, positive ? 1u : 0u, &value);
            return value;
        }

        private static unsafe PadControl[] ForNative(PadButton[] buttons, PadAxis[] axes)
        {
            uint[] b = Array.ConvertAll(buttons, x => (uint)x), a = Array.ConvertAll(axes, x => (uint)x), out_ = new uint[64];
            long n;
            fixed (uint* pb = b, pa = a, po = out_) n = EndymionNative.PadControlsFor(pb, (nuint)b.Length, pa, (nuint)a.Length, po, (nuint)out_.Length);
            return out_.Take((int)n).Select(c => (PadControl)c).ToArray();
        }

        // Galaxia's PadControls stays in C# for the frontends, so the library's rules are still held to it live.
        [Fact]
        public void The_pads_rules_give_the_same_value_for_every_axis_reading_and_control()
        {
            double[] readings = { 0, -0.0, 1, -1, 0.5, -0.5, 2, -2, 1e-300, double.NaN, double.PositiveInfinity, double.NegativeInfinity, double.Epsilon, 0.999999999, -1.0000000001, 32767.0 / 32767 };
            var random = new Random(16);
            var different = new List<string>();
            for (int axis = 0; axis < 8; axis++)
            {
                foreach (double reading in readings.Concat(Enumerable.Range(0, 200).Select(_ => (random.NextDouble() - 0.5) * 4)))
                {
                    var masks = new List<uint> { 0, uint.MaxValue, 1u << 12, 1u << 13, 1u << 16, 1u << 17, 1u << 18, 1u << 19, 1u << 20 | 1u << 21, 1u << 22, 1u << 23, 3u << 18 };
                    masks.AddRange(Enumerable.Range(0, 20).Select(_ => (uint)random.Next() & 0xFFFFFF));
                    foreach (uint mask in masks)
                    {
                        double managed = PadControls.Resolve((PadAxis)axis, reading, c => (mask >> (int)c & 1) != 0);
                        if (Bits(managed) != Bits(ResolveNative((PadAxis)axis, reading, mask))) different.Add($"resolve {axis} {reading} {mask:X}");
                    }
                }
            }
            foreach (double reading in readings)
                foreach (bool negative in new[] { false, true })
                    foreach (bool positive in new[] { false, true })
                        if (Bits(PadControls.Combine(reading, negative, positive)) != Bits(CombineNative(reading, negative, positive))) different.Add($"combine {reading} {negative} {positive}");
            PadButton[] buttons = Enum.GetValues<PadButton>();
            PadAxis[] axes = Enum.GetValues<PadAxis>();
            for (int i = 0; i < 500; i++)
            {
                PadButton[] b = Enumerable.Range(0, random.Next(0, 20)).Select(_ => buttons[random.Next(buttons.Length)]).ToArray();
                PadAxis[] a = Enumerable.Range(0, random.Next(0, 8)).Select(_ => axes[random.Next(axes.Length)]).ToArray();
                if (!PadControls.For(b, a).SequenceEqual(ForNative(b, a))) different.Add($"for [{string.Join(",", b)}] [{string.Join(",", a)}]");
            }
            AssertNone(different, "answers");
        }

        // The seats as the pads in them, named by their place in the pool.
        private static string Seats(PlayerSlots slots, List<ConnectedPad> pool) =>
            string.Join(" ", Enumerable.Range(1, PlayerSlots.MaxPlayers).Select(p => slots.SeatOf(p) is { } pad ? $"{pool.IndexOf(pad)}{(pad.IsOpen ? "" : "x")}" : "-"));

        [Fact]
        public void The_seats_take_trade_keep_and_let_go_of_pads_as_recorded()
        {
            var golden = Golden("Seats");
            var random = new Random(8);
            string[] names = { "Pad", "Pad", "Pro Controller", "DualSense", "日本 pad" };
            string?[] paths = { null, "/dev/input/a", "/dev/input/b", "", "/dev/input/a" };
            for (int run = 0; run < 150; run++)
            {
                SimulatedPads devices = new SimulatedPads();
                using GamepadManager manager = new GamepadManager(new GamepadBindingMap(), start: true, devices);
                var slots = new PlayerSlots();
                int changes = 0;
                slots.Changed += () => changes++;
                var pool = new List<ConnectedPad>();
                var simulated = new List<SimulatedPad>();
                for (int step = 0; step < 80; step++)
                {
                    string what;
                    string answer = "";
                    switch (random.Next(9))
                    {
                        case 0:
                        case 1:
                            SimulatedPad pad = new SimulatedPad();
                            pad.Name = names[random.Next(names.Length)];
                            pad.Path = paths[random.Next(paths.Length)];
                            if (random.Next(4) == 0) pad.Guid = random.Next(2) == 0 ? "shared" : "";
                            simulated.Add(pad);
                            devices.Connect(pad);
                            manager.Poll();
                            foreach (ConnectedPad opened in manager.Pads) if (!pool.Contains(opened)) pool.Add(opened);
                            what = "connect";
                            break;
                        case 2:
                            if (simulated.Count == 0) continue;
                            devices.Disconnect(simulated[random.Next(simulated.Count)]);
                            manager.Poll();
                            what = "disconnect";
                            break;
                        case 3:
                        case 4:
                            if (pool.Count == 0) continue;
                            ConnectedPad seat = pool[random.Next(pool.Count)];
                            answer = slots.Seat(seat).ToString();
                            what = $"seat {pool.IndexOf(seat)}";
                            break;
                        case 5:
                        case 6:
                            if (pool.Count == 0) continue;
                            ConnectedPad move = pool[random.Next(pool.Count)];
                            int to = random.Next(-1, PlayerSlots.MaxPlayers + 2);
                            answer = Outcome(() => { slots.Move(move, to); return "ok"; });
                            what = $"move {pool.IndexOf(move)} to {to}";
                            break;
                        case 7:
                            int forget = random.Next(-1, PlayerSlots.MaxPlayers + 2);
                            slots.Forget(forget);
                            what = $"forget {forget}";
                            break;
                        default:
                            if (random.Next(10) == 0) slots.Clear();
                            answer = slots.Highest.ToString();
                            what = "highest";
                            break;
                    }
                    golden.AddChunked("step", $"{what}: {answer} [{Seats(slots, pool)}] {changes}", 64);
                }
            }
            golden.Check(100);
        }

        private sealed class Log
        {
            public readonly List<string> Calls = new();
            public Action<int, PadButton, bool> Button => (p, b, h) => Calls.Add($"b {p} {b} {h}");
            public Action<int, PadAxis, double> Axis => (p, a, v) => Calls.Add($"a {p} {a} {Bits(v)}");
            public Action<int, bool> Connected => (p, c) => Calls.Add($"c {p} {c}");
        }

        [Fact]
        public void The_router_tells_the_core_the_recorded_things_in_the_recorded_order()
        {
            var golden = Golden("Router");
            var random = new Random(4);
            PadAxis[] allAxes = Enum.GetValues<PadAxis>();
            SDL.GamepadButton[] sdlButtons = Enum.GetValues<SDL.GamepadButton>().Where(b => b != SDL.GamepadButton.Invalid && b != SDL.GamepadButton.Count).ToArray();
            SDL.GamepadAxis[] sdlAxes = Enum.GetValues<SDL.GamepadAxis>().Where(a => a != SDL.GamepadAxis.Invalid && a != SDL.GamepadAxis.Count).ToArray();
            for (int run = 0; run < 120; run++)
            {
                var simulated = Enumerable.Range(0, random.Next(0, 10)).Select(i => { SimulatedPad pad = new SimulatedPad(); pad.Name = $"Pad {i % 4}"; return pad; }).ToList();
                SimulatedPads devices = SimulatedPads.With(simulated.ToArray());
                if (simulated.Count == 0) devices = new SimulatedPads();
                using GamepadManager manager = new GamepadManager(new GamepadBindingMap(), start: true, devices);
                bool withConnected = random.Next(4) != 0;
                Log log = new();
                bool[] keys = new bool[Enum.GetValues<PadControl>().Length];
                var router = new PortRouter(manager, log.Button, log.Axis, withConnected ? log.Connected : null) { KeyboardHeld = c => keys[(int)c] };
                int told = 0;
                for (int step = 0; step < 70; step++)
                {
                    string what = "";
                    switch (random.Next(16))
                    {
                        case 0:
                            int ports = random.Next(0, 11);
                            PadAxis[] axes = Enumerable.Range(0, random.Next(0, 7)).Select(_ => allAxes[random.Next(allAxes.Length)]).ToArray();
                            router.Reset(ports, axes);
                            what = $"reset {ports} [{string.Join(",", axes)}]";
                            break;
                        case 1: int to = random.Next(-2, 11); router.Resize(to); what = $"resize {to}"; break;
                        case 2: int kb = random.Next(0, 11); router.KeyboardPlayer = kb; what = $"keyboard {kb}"; break;
                        case 3: router.MirrorPlayer1ToPlayer2 = !router.MirrorPlayer1ToPlayer2; what = "mirror"; break;
                        case 4: keys[random.Next(keys.Length)] ^= true; router.KeysChanged(); what = "key"; break;
                        case 5: manager.FirstControllerOnly = !manager.FirstControllerOnly; what = "first only"; break;
                        case 6:
                            if (simulated.Count == 0) break;
                            SimulatedPad unplugged = simulated[random.Next(simulated.Count)];
                            if (random.Next(2) == 0) devices.Disconnect(unplugged); else devices.Connect(unplugged);
                            manager.Poll();
                            what = "plug";
                            break;
                        case 7:
                            if (manager.Pads.Count == 0) break;
                            manager.Assign(manager.Pads[random.Next(manager.Pads.Count)], random.Next(0, 9));
                            what = "assign";
                            break;
                        case 8:
                            int forget = random.Next(1, 9);
                            manager.Players.Forget(forget);
                            what = "forget";
                            break;
                        case 9:
                        case 10:
                            if (simulated.Count == 0) break;
                            SimulatedPad axisPad = simulated[random.Next(simulated.Count)];
                            double value = random.Next(4) switch { 0 => 1, 1 => -1, 2 => 0, _ => random.NextDouble() * 2 - 1 };
                            axisPad.SetAxis(sdlAxes[random.Next(sdlAxes.Length)], value);
                            what = "stick";
                            break;
                        default:
                            if (simulated.Count > 0)
                            {
                                SimulatedPad pressed = simulated[random.Next(simulated.Count)];
                                SDL.GamepadButton button = sdlButtons[random.Next(sdlButtons.Length)];
                                if (pressed.IsHeld(button)) pressed.Release(button); else pressed.Press(button);
                            }
                            manager.AnalogStickAsDpad = random.Next(5) != 0;
                            router.PollPads();
                            what = "poll";
                            break;
                    }
                    string queries = $"{router.Ports} {router.KeyboardPlayer} {router.MirrorPlayer1ToPlayer2} " + string.Concat(Enumerable.Range(-1, 13).Select(p => router.Connected(p) ? "1" : "0")) + " "
                        + string.Concat(Enumerable.Range(0, 10).SelectMany(p => Enum.GetValues<PadButton>().Select(b => router.PadHeld(p, b) ? "1" : "0")));
                    golden.AddChunked("step", $"{what}: {string.Join(" / ", log.Calls.Skip(told))} | {queries}", 64);
                    told = log.Calls.Count;
                }
            }
            golden.Check(100);
        }

        [Fact]
        public void The_library_has_every_export_this_half_calls_and_no_other()
        {
            string header = System.IO.File.ReadAllText(System.IO.Path.Combine(EmuSen.Galaxia.ConfigRoot.Directory, "Platform", "include", "emusen_platform.h"));
            foreach (string export in EndymionNative.Exports) Assert.Contains(export + "(", header);
            // Interface 43's export serves the Qt front end's game pads; the C# half resolves its bindings itself (EmuSen_RustPlatform.md §50).
            string[] qtOnly = { "emusen_endymion_bindings_bound" };
            foreach (string export in qtOnly) Assert.Contains(export + "(", header);
            Assert.Equal(EndymionNative.Exports.Count(e => e.StartsWith("emusen_endymion_")) + qtOnly.Length, System.Text.RegularExpressions.Regex.Matches(header, @"^\w+ \*?emusen_endymion_\w+\(", System.Text.RegularExpressions.RegexOptions.Multiline).Count);
        }
    }
}
