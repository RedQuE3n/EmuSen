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

        // Chains of the operations that read and write the accumulator, so a carry kept wrongly in its three thirds compounds instead of being rewritten from the array each time - see Mars_RspVector.md §15.
        [Fact]
        public void Chains_of_accumulator_operations_agree_with_the_unit_they_are_built_from()
        {
            int[] family = { 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x14, 0x1D };
            var failures = new System.Collections.Generic.List<string>();

            for (int chain = 0; chain < 4000; chain++)
            {
                var seed = new Random(chain);
                var plain = Started(false, out var plainRsp);
                var wide = Started(true, out var wideRsp);

                ushort[] vector = new ushort[32 * 8];
                ulong[] accumulator = new ulong[8];
                for (int i = 0; i < vector.Length; i++) vector[i] = Value(seed);
                for (int i = 0; i < accumulator.Length; i++) accumulator[i] = ((ulong)Value(seed) << 32 | (ulong)Value(seed) << 16 | Value(seed)) & 0xFFFF_FFFF_FFFF;

                foreach (var rsp in new[] { plainRsp, wideRsp })
                {
                    vector.CopyTo(rsp.Vector, 0);
                    accumulator.CopyTo(rsp.Accumulator, 0);
                    rsp.AccumulatorWritten();
                }

                for (int link = 0; link < 10; link++)
                {
                    int function = family[seed.Next(family.Length)];
                    uint instruction = Vector(function, vd: 1 + seed.Next(4), vt: 1 + seed.Next(4), vs: 1 + seed.Next(4), function == 0x1D ? 8 + seed.Next(3) : seed.Next(16));
                    Step(plain, plainRsp, instruction);
                    Step(wide, wideRsp, instruction);
                }

                string one = State(plainRsp), two = State(wideRsp);
                if (one != two && failures.Count < 6) failures.Add($"chain {chain}:\n  by lanes {one}\n  by vector {two}");
            }

            Assert.Empty(failures);
        }

        // The quad load and store at every address near the end of data memory and at several elements, against their definition byte by byte: the whole-register case is one vector move, and it must stop where the bytes would wrap - see Mars_RspVector.md §15.
        [Fact]
        public void A_quad_load_and_store_move_the_bytes_their_definition_moves_up_to_the_end_of_memory()
        {
            var seed = new Random(11);

            foreach (bool store in new[] { false, true })
            {
                for (uint address = 0xFD0; address <= 0xFFF; address++)
                {
                    foreach (int element in new[] { 0, 1, 2, 15 })
                    {
                        var bus = Started(true, out var rsp);
                        seed.NextBytes(bus.SpDmem);
                        for (int i = 0; i < rsp.Vector.Length; i++) rsp.Vector[i] = (ushort)seed.Next(0x10000);

                        byte[] memory = (byte[])bus.SpDmem.Clone();
                        byte[] register = new byte[16];
                        for (int b = 0; b < 16; b++) register[b] = (byte)(b % 2 == 0 ? rsp.Vector[5 * 8 + b / 2] >> 8 : rsp.Vector[5 * 8 + b / 2]);

                        int count = 16 - (int)(address & 0xF);
                        if (store) for (int i = 0; i < count; i++) memory[(address + i) & 0xFFF] = register[(element + i) & 0xF];
                        else for (int i = 0; i < Math.Min(16 - element, count); i++) register[element + i] = memory[(address + i) & 0xFFF];

                        // The base register holds the address and the offset is zero.
                        rsp.Gpr[1] = address;
                        Step(bus, rsp, (store ? 0xE800_0000u : 0xC800_0000u) | (1u << 21) | (5u << 16) | (4u << 11) | ((uint)element << 7));

                        for (int b = 0; b < 16; b++)
                        {
                            byte got = (byte)(b % 2 == 0 ? rsp.Vector[5 * 8 + b / 2] >> 8 : rsp.Vector[5 * 8 + b / 2]);
                            Assert.True(register[b] == got, $"{(store ? "store" : "load")} at {address:X3} element {element}: register byte {b} is {got:X2}, not {register[b]:X2}");
                        }

                        // The instruction itself was written over the start of instruction memory, not data memory, so every data byte is comparable.
                        Assert.Equal(memory, bus.SpDmem);
                    }
                }
            }
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
                rsp.AccumulatorWritten();
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

            rsp.WidenAccumulator();
            text.Append(" acc=");
            foreach (ulong value in rsp.Accumulator) text.Append($"{value:X12},");

            return text.Append($" vco={rsp.Vco:X4} vcc={rsp.Vcc:X4} vce={rsp.Vce:X2}").ToString();
        }

        private static uint Vector(int function, int vd, int vt, int vs, int selector) =>
            0x4A00_0000u | ((uint)selector << 21) | ((uint)vt << 16) | ((uint)vs << 11) | ((uint)vd << 6) | (uint)function;
    }
}
