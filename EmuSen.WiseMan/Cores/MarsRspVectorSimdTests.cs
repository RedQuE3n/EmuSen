using System;
using System.Buffers.Binary;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Cores
{
    // The vector unit's eight-lane arithmetic against the element-by-element unit that is its reference - see Mars_RspVector.md §14.
    public class MarsRspVectorSimdTests
    {
        private const int Functions = 64;
        private const int Selectors = 16;

        // The values a lane is given: the boundaries every clamp and every carry turns on, then whatever the generator says.
        private static readonly ushort[] Corners = { 0x0000, 0x0001, 0x7FFF, 0x8000, 0x8001, 0xFFFF, 0x4000, 0xC000 };

        [Fact]
        public void Every_function_and_selector_agrees_with_the_unit_it_is_built_from()
        {
            var failures = new System.Collections.Generic.List<string>();
            int cases = 0;

            for (int function = 0; function < Functions; function++)
            {
                for (int selector = 0; selector < Selectors; selector++)
                {
                    for (int round = 0; round < 6; round++)
                    {
                        cases++;
                        uint instruction = Vector(function, vd: 3, vt: 2, vs: 1, selector);
                        var seed = new Random(function * 4096 + selector * 16 + round);

                        string? difference = Compare(instruction, seed, aliased: round == 5 ? 1 : 0);
                        if (difference != null && failures.Count < 12) failures.Add($"function {function:X2} selector {selector} round {round}: {difference}");
                    }
                }
            }

            Assert.Equal(6144, cases);
            Assert.Empty(failures);
        }

        // The two halves of a double-precision reciprocal in sequence, since the high half leaves state for the low one.
        [Fact]
        public void A_double_precision_reciprocal_agrees_across_its_two_instructions()
        {
            foreach (int pair in new[] { 0x32, 0x36 })
            {
                foreach (ushort high in Corners)
                {
                    foreach (ushort low in Corners)
                    {
                        var plain = Started(false, out var plainRsp);
                        var wide = Started(true, out var wideRsp);

                        foreach (var rsp in new[] { plainRsp, wideRsp })
                        {
                            rsp.Vector[2 * 8 + 0] = high;
                            rsp.Vector[2 * 8 + 1] = low;
                        }

                        Step(plain, plainRsp, Vector(pair, vd: 3, vt: 2, vs: 0, selector: 8));
                        Step(wide, wideRsp, Vector(pair, vd: 3, vt: 2, vs: 0, selector: 8));
                        Step(plain, plainRsp, Vector(pair - 1, vd: 4, vt: 2, vs: 0, selector: 9));
                        Step(wide, wideRsp, Vector(pair - 1, vd: 4, vt: 2, vs: 0, selector: 9));

                        Assert.Equal(State(plainRsp), State(wideRsp));
                    }
                }
            }
        }

        // Written before the gain was measured: what the reference says about the one instruction the two paths order differently.
        [Fact]
        public void A_destination_that_is_also_a_source_is_read_before_it_is_written()
        {
            var plain = Started(false, out var plainRsp);
            var wide = Started(true, out var wideRsp);

            foreach (var rsp in new[] { plainRsp, wideRsp })
            {
                for (int i = 0; i < 8; i++)
                {
                    rsp.Vector[1 * 8 + i] = (ushort)(0x1000 + i);
                    rsp.Vector[2 * 8 + i] = (ushort)(0x0100 * (i + 1));
                }
            }

            uint or = Vector(0x2A, vd: 1, vt: 2, vs: 1, selector: 4);
            Step(plain, plainRsp, or);
            Step(wide, wideRsp, or);

            Assert.Equal(State(plainRsp), State(wideRsp));
            Assert.NotEqual(0x1000 | 0x0100, plainRsp.Vector[1 * 8 + 1]);
        }

        private static string? Compare(uint instruction, Random seed, int aliased)
        {
            var plain = Started(false, out var plainRsp);
            var wide = Started(true, out var wideRsp);

            ushort[] vector = new ushort[32 * 8];
            ulong[] accumulator = new ulong[8];

            for (int i = 0; i < vector.Length; i++) vector[i] = Value(seed);
            for (int i = 0; i < accumulator.Length; i++) accumulator[i] = ((ulong)Value(seed) << 32 | (ulong)Value(seed) << 16 | Value(seed)) & 0xFFFF_FFFF_FFFF;

            ushort vco = Value(seed), vcc = Value(seed);
            byte vce = (byte)Value(seed);

            // One round names the same register as source and destination, which is where reading before writing shows - see §2.
            uint used = aliased == 0 ? instruction : Vector((int)(instruction & 0x3F), vd: 1, vt: 2, vs: 1, (int)((instruction >> 21) & 0xF));

            foreach (var rsp in new[] { plainRsp, wideRsp })
            {
                vector.CopyTo(rsp.Vector, 0);
                accumulator.CopyTo(rsp.Accumulator, 0);
                rsp.Vco = vco;
                rsp.Vcc = vcc;
                rsp.Vce = vce;
            }

            Step(plain, plainRsp, used);
            Step(wide, wideRsp, used);

            string one = State(plainRsp), two = State(wideRsp);
            return one == two ? null : $"\n  by lanes {one}\n  by vector {two}";
        }

        private static ushort Value(Random seed) =>
            seed.Next(3) == 0 ? Corners[seed.Next(Corners.Length)] : (ushort)seed.Next(0x10000);

        private static MemoryBus Started(bool simd, out EmuSen.Cores.Nintendo.Mars.Rsp.Rsp rsp)
        {
            var bus = new MemoryBus();
            rsp = bus.Sp.Processor;
            rsp.UseSimd = simd;
            return bus;
        }

        // One instruction, run where the processor's own fetch and dispatch reach it.
        private static void Step(MemoryBus bus, EmuSen.Cores.Nintendo.Mars.Rsp.Rsp rsp, uint instruction)
        {
            BinaryPrimitives.WriteUInt32BigEndian(bus.SpImem.AsSpan(0), instruction);
            rsp.Start(0);
            rsp.Step();
            rsp.Halted = true;
        }

        private static string State(EmuSen.Cores.Nintendo.Mars.Rsp.Rsp rsp)
        {
            var text = new System.Text.StringBuilder();
            for (int register = 1; register <= 4; register++)
            {
                text.Append($" v{register}=");
                for (int i = 0; i < 8; i++) text.Append($"{rsp.Vector[register * 8 + i]:X4},");
            }

            text.Append(" acc=");
            foreach (ulong value in rsp.Accumulator) text.Append($"{value:X12},");

            return text.Append($" vco={rsp.Vco:X4} vcc={rsp.Vcc:X4} vce={rsp.Vce:X2}").ToString();
        }

        private static uint Vector(int function, int vd, int vt, int vs, int selector) =>
            0x4A00_0000u | ((uint)selector << 21) | ((uint)vt << 16) | ((uint)vs << 11) | ((uint)vd << 6) | (uint)function;
    }
}
