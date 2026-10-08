using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Input;
using SDL3;

namespace EmuSen.WiseMan.Endymion
{
    // Endymion's C# rules and the platform library's, asked the same things side by side and held to the bit - see EmuSen_RustPlatform.md §12.5.
    public class EndymionParityTests
    {
        public EndymionParityTests() => Assert.True(EndymionNative.Ready, EndymionNative.Report);

        private static string Bits(double value) => BitConverter.DoubleToInt64Bits(value).ToString("X16");

        private static void AssertNone(List<string> different, string what) =>
            Assert.True(different.Count == 0, $"{different.Count} {what} differ:\n{string.Join("\n", different.Take(20))}");

        // What a call came to: its answer, or the exception it threw with its words.
        private static string Outcome(Func<string> call)
        {
            try { return call(); }
            catch (Exception e) { return $"{e.GetType().Name}: {e.Message}"; }
        }

        [Fact]
        public void The_switch_is_what_the_variable_says()
        {
            Assert.Equal(Environment.GetEnvironmentVariable(EndymionNative.Variable) == "1", EndymionNative.Active);
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
        public void The_resampler_makes_the_same_samples_from_every_stream_at_every_ratio()
        {
            var random = new Random(32000);
            var different = new List<string>();
            for (int run = 0; run < 400; run++)
            {
                var managed = new LinearResampler(native: false);
                var native = new LinearResampler(native: true);
                for (int call = 0; call < 30; call++)
                {
                    short[] input = Stream(random, random.Next(0, 300));
                    double ratio = random.Next(3) == 0 ? Ratios[random.Next(Ratios.Length)] : 1 + (random.NextDouble() - 0.5) * 0.02;
                    if (random.Next(25) == 0) { managed.Reset(); native.Reset(); }
                    string a = Outcome(() => string.Join(",", managed.Resample(input, ratio)));
                    string b = Outcome(() => string.Join(",", native.Resample(input, ratio)));
                    if (a != b) different.Add($"run {run} call {call} ratio {Bits(ratio)} length {input.Length}: {a[..Math.Min(80, a.Length)]} and {b[..Math.Min(80, b.Length)]}");
                }
            }
            AssertNone(different, "runs");
        }

        [Fact]
        public void A_ratio_and_an_input_are_refused_alike_and_only_the_library_refuses_what_the_csharp_never_returns_from()
        {
            foreach (double ratio in new[] { 0.0, -0.0, -1.0, double.NegativeInfinity, double.MinValue })
                foreach (short[]? input in new short[]?[] { new short[] { 1, 2, 3, 4 }, Array.Empty<short>(), null })
                {
                    string managed = Outcome(() => string.Join(",", new LinearResampler(native: false).Resample(input!, ratio)));
                    string native = Outcome(() => string.Join(",", new LinearResampler(native: true).Resample(input!, ratio)));
                    Assert.Equal(managed, native);
                    Assert.StartsWith("ArgumentOutOfRangeException: Resample ratio must be positive.", native);
                }
            Assert.Equal(Outcome(() => string.Join(",", new LinearResampler(native: false).Resample(null!, 1.0))), Outcome(() => string.Join(",", new LinearResampler(native: true).Resample(null!, 1.0))));
            Assert.Empty(new LinearResampler(native: true).Resample(Array.Empty<short>(), 1.0));
            Assert.Empty(new LinearResampler(native: true).Resample(new short[] { 5 }, 1.0));
            foreach (double ratio in new[] { double.NaN, double.PositiveInfinity })
                Assert.Equal("ArgumentOutOfRangeException: Resample ratio must be finite. (Parameter 'ratio')", Outcome(() => string.Join(",", new LinearResampler(native: true).Resample(new short[] { 1, 2 }, ratio))));
        }

        private static string State(DynamicRateControl control) =>
            $"{control.TargetQueuedFrames} {Bits(control.MaxDeviation)} {Bits(control.SheddingEntryFactor)} {Bits(control.SheddingExitFactor)} {Bits(control.NominalRatio)} {Bits(control.LastRatio)} {control.SheddingEvents} {control.IsShedding} {control.TotalInputFrames} {control.TotalOutputFrames}";

        [Fact]
        public void The_rate_control_steers_sheds_and_counts_the_same_through_every_queue()
        {
            var random = new Random(48000);
            var different = new List<string>();
            int[] targets = { 0, 1, 64, 1024, 8192, -5, 32000 };
            for (int run = 0; run < 300; run++)
            {
                int target = targets[random.Next(targets.Length)];
                var managed = new DynamicRateControl(target, native: false);
                var native = new DynamicRateControl(target, native: true);
                int queued = Math.Max(0, target);
                for (int step = 0; step < 60; step++)
                {
                    switch (random.Next(14))
                    {
                        case 0: double d = random.Next(2) == 0 ? 0.005 : random.NextDouble() * 0.2; managed.MaxDeviation = d; native.MaxDeviation = d; break;
                        case 1: double e = 1 + random.NextDouble() * 4; managed.SheddingEntryFactor = e; native.SheddingEntryFactor = e; break;
                        case 2: double x = random.NextDouble() * 3; managed.SheddingExitFactor = x; native.SheddingExitFactor = x; break;
                        case 3: double n = random.Next(2) == 0 ? 1.0 : 0.9 + random.NextDouble() * 0.2; managed.NominalRatio = n; native.NominalRatio = n; break;
                        case 4: int t = targets[random.Next(targets.Length)]; managed.TargetQueuedFrames = t; native.TargetQueuedFrames = t; break;
                        case 5: managed.Reset(); native.Reset(); break;
                        default:
                            queued = random.Next(4) == 0 ? random.Next(-100, Math.Max(1, target) * 5) : Math.Max(0, queued + random.Next(-300, 300));
                            short[] input = Stream(random, random.Next(0, 600));
                            string a = Outcome(() => string.Join(",", managed.Process(input, queued)));
                            string b = Outcome(() => string.Join(",", native.Process(input, queued)));
                            if (a != b) different.Add($"run {run} step {step}: output differs");
                            if (Bits(managed.ComputeRatio(queued)) != Bits(native.ComputeRatio(queued))) different.Add($"run {run} step {step}: ratio for {queued}");
                            break;
                    }
                    if (State(managed) != State(native)) different.Add($"run {run} step {step}: {State(managed)} and {State(native)}");
                }
            }
            AssertNone(different, "steps");

            var m = new DynamicRateControl(100, native: false);
            var v = new DynamicRateControl(100, native: true);
            Assert.Equal(Outcome(() => string.Join(",", m.Process(null!, 250))), Outcome(() => string.Join(",", v.Process(null!, 250))));
            Assert.Equal(State(m), State(v));
            m.NominalRatio = v.NominalRatio = -1;
            Assert.Equal(Outcome(() => string.Join(",", m.Process(new short[] { 1, 2, 3, 4 }, 100))), Outcome(() => string.Join(",", v.Process(new short[] { 1, 2, 3, 4 }, 100))));
            Assert.Equal(State(m), State(v));
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

        // The seats of each implementation as the pads in them, named by their place in the pool.
        private static string Seats(PlayerSlots slots, List<ConnectedPad> pool) =>
            string.Join(" ", Enumerable.Range(1, PlayerSlots.MaxPlayers).Select(p => slots.SeatOf(p) is { } pad ? $"{pool.IndexOf(pad)}{(pad.IsOpen ? "" : "x")}" : "-"));

        [Fact]
        public void The_seats_take_trade_keep_and_let_go_of_pads_the_same_way()
        {
            var random = new Random(8);
            var different = new List<string>();
            string[] names = { "Pad", "Pad", "Pro Controller", "DualSense", "日本 pad" };
            string?[] paths = { null, "/dev/input/a", "/dev/input/b", "", "/dev/input/a" };
            for (int run = 0; run < 150; run++)
            {
                var devices = new SimulatedPads();
                var manager = new GamepadManager(new GamepadBindingMap(), start: true, devices);
                var managed = new PlayerSlots(native: false);
                var native = new PlayerSlots(native: true);
                int managedChanges = 0, nativeChanges = 0;
                managed.Changed += () => managedChanges++;
                native.Changed += () => nativeChanges++;
                var pool = new List<ConnectedPad>();
                var simulated = new List<SimulatedPad>();
                for (int step = 0; step < 80; step++)
                {
                    string what;
                    string answerManaged = "", answerNative = "";
                    switch (random.Next(9))
                    {
                        case 0:
                        case 1:
                            var pad = new SimulatedPad { Name = names[random.Next(names.Length)], Path = paths[random.Next(paths.Length)] };
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
                            answerManaged = managed.Seat(seat).ToString();
                            answerNative = native.Seat(seat).ToString();
                            what = $"seat {pool.IndexOf(seat)}";
                            break;
                        case 5:
                        case 6:
                            if (pool.Count == 0) continue;
                            ConnectedPad move = pool[random.Next(pool.Count)];
                            int to = random.Next(-1, PlayerSlots.MaxPlayers + 2);
                            answerManaged = Outcome(() => { managed.Move(move, to); return "ok"; });
                            answerNative = Outcome(() => { native.Move(move, to); return "ok"; });
                            what = $"move {pool.IndexOf(move)} to {to}";
                            break;
                        case 7:
                            int forget = random.Next(-1, PlayerSlots.MaxPlayers + 2);
                            managed.Forget(forget);
                            native.Forget(forget);
                            what = $"forget {forget}";
                            break;
                        default:
                            if (random.Next(10) == 0) { managed.Clear(); native.Clear(); }
                            answerManaged = managed.Highest.ToString();
                            answerNative = native.Highest.ToString();
                            what = "highest";
                            break;
                    }
                    string a = Seats(managed, pool), b = Seats(native, pool);
                    if (a != b || managedChanges != nativeChanges || answerManaged != answerNative)
                        different.Add($"run {run} step {step} {what}: {answerManaged} [{a}] {managedChanges} and {answerNative} [{b}] {nativeChanges}");
                }
            }
            AssertNone(different, "steps");
        }

        private sealed class Log
        {
            public readonly List<string> Calls = new();
            public Action<int, PadButton, bool> Button => (p, b, h) => Calls.Add($"b {p} {b} {h}");
            public Action<int, PadAxis, double> Axis => (p, a, v) => Calls.Add($"a {p} {a} {Bits(v)}");
            public Action<int, bool> Connected => (p, c) => Calls.Add($"c {p} {c}");
        }

        [Fact]
        public void The_router_tells_the_core_the_same_things_in_the_same_order()
        {
            var random = new Random(4);
            var different = new List<string>();
            PadAxis[] allAxes = Enum.GetValues<PadAxis>();
            SDL.GamepadButton[] sdlButtons = Enum.GetValues<SDL.GamepadButton>().Where(b => b != SDL.GamepadButton.Invalid && b != SDL.GamepadButton.Count).ToArray();
            SDL.GamepadAxis[] sdlAxes = Enum.GetValues<SDL.GamepadAxis>().Where(a => a != SDL.GamepadAxis.Invalid && a != SDL.GamepadAxis.Count).ToArray();
            for (int run = 0; run < 120; run++)
            {
                var simulated = Enumerable.Range(0, random.Next(0, 10)).Select(i => new SimulatedPad { Name = $"Pad {i % 4}" }).ToList();
                SimulatedPads devices = SimulatedPads.With(simulated.ToArray());
                var manager = new GamepadManager(new GamepadBindingMap(), start: true, devices);
                bool withConnected = random.Next(4) != 0;
                Log managedLog = new(), nativeLog = new();
                bool[] keys = new bool[Enum.GetValues<PadControl>().Length];
                var managed = new PortRouter(manager, managedLog.Button, managedLog.Axis, withConnected ? managedLog.Connected : null, native: false) { KeyboardHeld = c => keys[(int)c] };
                var native = new PortRouter(manager, nativeLog.Button, nativeLog.Axis, withConnected ? nativeLog.Connected : null, native: true) { KeyboardHeld = c => keys[(int)c] };
                for (int step = 0; step < 70; step++)
                {
                    string what = "";
                    switch (random.Next(16))
                    {
                        case 0:
                            int ports = random.Next(0, 11);
                            PadAxis[] axes = Enumerable.Range(0, random.Next(0, 7)).Select(_ => allAxes[random.Next(allAxes.Length)]).ToArray();
                            managed.Reset(ports, axes);
                            native.Reset(ports, axes);
                            what = $"reset {ports} [{string.Join(",", axes)}]";
                            break;
                        case 1: int to = random.Next(-2, 11); managed.Resize(to); native.Resize(to); what = $"resize {to}"; break;
                        case 2: int kb = random.Next(0, 11); managed.KeyboardPlayer = kb; native.KeyboardPlayer = kb; what = $"keyboard {kb}"; break;
                        case 3: managed.MirrorPlayer1ToPlayer2 = native.MirrorPlayer1ToPlayer2 = !managed.MirrorPlayer1ToPlayer2; what = "mirror"; break;
                        case 4: keys[random.Next(keys.Length)] ^= true; managed.KeysChanged(); native.KeysChanged(); what = "key"; break;
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
                            managed.PollPads();
                            native.PollPads();
                            what = "poll";
                            break;
                    }
                    if (!managedLog.Calls.SequenceEqual(nativeLog.Calls))
                    {
                        int at = managedLog.Calls.Zip(nativeLog.Calls).TakeWhile(p => p.First == p.Second).Count();
                        different.Add($"run {run} step {step} {what}: call {at} of {managedLog.Calls.Count} and {nativeLog.Calls.Count}: {managedLog.Calls.ElementAtOrDefault(at)} and {nativeLog.Calls.ElementAtOrDefault(at)}");
                        break;
                    }
                    string Queries(PortRouter r) => $"{r.Ports} {r.KeyboardPlayer} {r.MirrorPlayer1ToPlayer2} " + string.Concat(Enumerable.Range(-1, 13).Select(p => r.Connected(p) ? "1" : "0")) + " "
                        + string.Concat(Enumerable.Range(0, 10).SelectMany(p => Enum.GetValues<PadButton>().Select(b => r.PadHeld(p, b) ? "1" : "0")));
                    if (Queries(managed) != Queries(native)) { different.Add($"run {run} step {step} {what}: {Queries(managed)} and {Queries(native)}"); break; }
                }
            }
            AssertNone(different, "runs");
        }

        [Fact]
        public void The_library_has_every_export_this_half_calls_and_no_other()
        {
            string header = System.IO.File.ReadAllText(System.IO.Path.Combine(EmuSen.Galaxia.ConfigRoot.Managed.Directory, "Platform", "include", "emusen_platform.h"));
            foreach (string export in EndymionNative.Exports) Assert.Contains(export + "(", header);
            Assert.Equal(EndymionNative.Exports.Count(e => e.StartsWith("emusen_endymion_")), System.Text.RegularExpressions.Regex.Matches(header, @"^\w+ \*?emusen_endymion_\w+\(", System.Text.RegularExpressions.RegexOptions.Multiline).Count);
        }
    }
}
