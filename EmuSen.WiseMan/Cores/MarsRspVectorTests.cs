using System;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The signal processor's vector unit, in the corpus's own vectors - see Mars_RspVector.md.
    public class MarsRspVectorTests
    {
        private const int Vmulf = 0x00;
        private const int Vrndp = 0x02;
        private const int Vmudh = 0x07;
        private const int Vmadl = 0x0C;
        private const int Vmadm = 0x0D;
        private const int Vmadh = 0x0F;
        private const int Vadd = 0x10;
        private const int Vsut = 0x12;
        private const int Vsar = 0x1D;
        private const int Vch = 0x25;
        private const int Vor = 0x2A;
        private const int Vrcp = 0x30;
        private const int Vrcpl = 0x31;
        private const int Vrcph = 0x32;
        private const int Vrsq = 0x34;
        private const int Vrsql = 0x35;
        private const int Vrsqh = 0x36;
        private const int Vnop = 0x37;

        private const int Byte = 0;
        private const int Double = 3;
        private const int Quad = 4;
        private const int Transposed = 11;

        private const int SelectorH0 = 4;
        private const int SelectorElement0 = 8;

        // The three control registers keep sixteen, sixteen and eight bits, and any index names one of them - see §3.
        [Theory]
        [InlineData(0, 0x1234_8678u, 0xFFFF_8678u)]
        [InlineData(1, 0x8765_8321u, 0xFFFF_8321u)]
        [InlineData(2, 0x1122_3384u, 0x0000_0084u)]
        [InlineData(3, 0x1122_3384u, 0x0000_0084u)]
        [InlineData(5, 0x0000_1234u, 0x0000_1234u)]
        [InlineData(30, 0x0000_0017u, 0x0000_0017u)]
        public void A_control_register_keeps_its_width_and_reads_back_sign_extended_from_sixteen_bits(int index, uint written, uint expected)
        {
            var bus = Loaded(a => Li(a, 1, written)
                .Word(Cop2(6, 1, index))
                .Word(Cop2(2, 16, index))
                .Sw(16, 0, 0x100)
                .Break());

            Run(bus);

            Assert.Equal(expected, bus.Read32(MemoryMap.SpDmemBase + 0x100));
        }

        // A transfer in drops the byte past the register's end, and a transfer out wraps to its start - see §3.
        [Fact]
        public void A_transfer_into_the_last_byte_stops_at_the_end_but_a_transfer_out_wraps()
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0xAABB, 0xCCDD, 0xEEFF, 0xABBA, 0xBCCB, 0xCDDC, 0xEFFE, 0xACCA);

            Load(bus, a => Li(a.Word(Lqv(5, 0, 0, 0)).Word(Lqv(6, 0, 0, 0)), 1, 0x1234_5678)
                .Word(Cop2(4, 1, 5, 15))
                .Word(Cop2(4, 1, 6, 14))
                .Word(Cop2(0, 16, 5, 15))
                .Word(Cop2(0, 17, 6, 0))
                .Word(Sqv(5, 0, 0x10, 0))
                .Word(Sqv(6, 0, 0x11, 0))
                .Sw(16, 0, 0x200).Sw(17, 0, 0x204)
                .Break());

            Run(bus);

            Assert.Equal(0xAC56, ReadElements(bus, 0x100)[7]);
            Assert.Equal(0xAABB, ReadElements(bus, 0x100)[0]);
            Assert.Equal(0x5678, ReadElements(bus, 0x110)[7]);
            Assert.Equal(0x0000_56AAu, bus.Read32(MemoryMap.SpDmemBase + 0x200));
            Assert.Equal(0xFFFF_AABBu, bus.Read32(MemoryMap.SpDmemBase + 0x204));
        }

        // A load that runs out of register stops, and a store that runs out of register wraps - see §4 and §5.
        [Fact]
        public void A_load_stops_at_the_end_of_the_register_but_a_store_wraps_inside_it()
        {
            var bus = new MarsBus();
            Counting(bus);
            Filled(bus, 0x200, 0xEE);

            Load(bus, a => Li(a, 4, 0x400)
                .Word(Lqv(1, 0, 0x20, 0))
                .Word(Transfer(0x32, Double, 1, 12, 4, 0))
                .Word(Sqv(1, 0, 0x30, 0))
                .Word(Lqv(2, 0, 0x04, 0))
                .Word(Transfer(0x3A, Double, 2, 12, 0, 4))
                .Break());

            Run(bus);

            Assert.Equal(new byte[] { 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0x20, 0x21, 0x22, 0x23 }, Bytes(bus, 0x300, 16));
            Assert.Equal(new byte[] { 0x4C, 0x4D, 0x4E, 0x4F, 0x40, 0x41, 0x42, 0x43 }, Bytes(bus, 0x400, 8));
        }

        // A quad load reads only as far as the end of the sixteen-byte region its address falls in - see §4.
        [Fact]
        public void A_quad_load_from_a_misaligned_address_reads_to_the_end_of_its_region()
        {
            var bus = new MarsBus();
            Counting(bus);
            Filled(bus, 0x200, 0xEE);

            Load(bus, a => Li(a, 4, 0x25)
                .Word(Lqv(1, 0, 0x20, 0))
                .Word(Transfer(0x32, Quad, 1, 0, 0, 4))
                .Word(Sqv(1, 0, 0x30, 0))
                .Break());

            Run(bus);

            Assert.Equal(new byte[] { 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE }, Bytes(bus, 0x300, 16));
        }

        // The transposed load gives one element to each of the eight registers in vt's group - see §4.
        [Fact]
        public void A_transposed_load_spreads_one_element_across_each_register_of_the_group()
        {
            var bus = new MarsBus();
            Counting(bus);
            Filled(bus, 0x200, 0xEE);

            Load(bus, a =>
            {
                for (int r = 8; r < 16; r++) a.Word(Lqv(r, 0, 0x20, 0));
                a.Word(Transfer(0x32, Transposed, 9, 0, 0, 0));
                for (int r = 8; r < 16; r++) a.Word(Sqv(r, 0, 0x30 + (r - 8), 0));
                return a.Break();
            });

            Run(bus);

            for (int r = 0; r < 8; r++)
            {
                ushort[] elements = ReadElements(bus, 0x300 + (uint)(r * 0x10));
                for (int i = 0; i < 8; i++) Assert.Equal(i == r ? (ushort)((2 * i << 8) | (2 * i + 1)) : (ushort)0xEEEE, elements[i]);
            }
        }

        // A selector shuffles vt, and every element is read before any is written, even into vt itself - see §2.
        [Fact]
        public void A_selected_source_is_read_whole_before_the_destination_it_shares_is_written()
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 1, 2, 3, 4, 5, 6, 7, 8);
            Elements(bus, 0x010, 0x10, 0, 0, 0, 0x20, 0, 0, 0);

            Load(bus, a => a
                .Word(Lqv(1, 0, 0, 0))
                .Word(Lqv(2, 0, 1, 0))
                .Word(Vector(Vor, 1, 1, 2, SelectorH0))
                .Word(Sqv(1, 0, 0x10, 0))
                .Break());

            Run(bus);

            // Written in place, elements 1-3 would read element 0 after it became 0x11 and all be 0x11.
            Assert.Equal(new ushort[] { 0x11, 1, 1, 1, 0x25, 5, 5, 5 }, ReadElements(bus, 0x100));
        }

        // The corpus's VMULF table, accumulator included - see §7.
        [Fact]
        public void A_fractional_multiply_rounds_into_the_accumulator_and_clamps_only_its_one_overflow()
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0x0000, 0x0000, 0x0000, 0xE000, 0x8001, 0x8000, 0x7FFF, 0x8000);
            Elements(bus, 0x010, 0x0000, 0x0001, 0xFFFF, 0xFFFF, 0x8000, 0x7FFF, 0x7FFF, 0x8000);

            Load(bus, a => Thirds(a
                .Word(Lqv(0, 0, 0, 0))
                .Word(Lqv(1, 0, 1, 0))
                .Word(Vector(Vmulf, 2, 0, 1, 0)))
                .Word(Sqv(2, 0, 0x10, 0))
                .Break());

            Run(bus);

            Assert.Equal(new ushort[] { 0, 0, 0, 0, 0x7FFF, 0x8001, 0x7FFE, 0x7FFF }, ReadElements(bus, 0x100));
            Assert.Equal(new ushort[] { 0, 0, 0, 0, 0, 0xFFFF, 0, 0 }, ReadElements(bus, 0x110));
            Assert.Equal(new ushort[] { 0, 0, 0, 0, 0x7FFF, 0x8001, 0x7FFE, 0x8000 }, ReadElements(bus, 0x120));
            Assert.Equal(new ushort[] { 0x8000, 0x8000, 0x8000, 0xC000, 0x8000, 0x8000, 0x8002, 0x8000 }, ReadElements(bus, 0x130));
        }

        // The corpus's overflow case: the accumulator wraps at forty-eight bits, and the clamp reads the wrapped value - see §6.
        [Fact]
        public void An_accumulating_multiply_wraps_at_forty_eight_bits_and_clamps_what_it_wrapped_to()
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0x7FFF, 0, 0, 0, 0, 0, 0, 0);

            Load(bus, a =>
            {
                a.Word(Lqv(0, 0, 0, 0)).Word(Lqv(1, 0, 0, 0));
                a.Word(Vector(Vmudh, 2, 0, 1, 0)).Word(Vector(Vmadh, 2, 0, 1, 0));
                for (int i = 0; i < 8; i++) a.Word(Vector(Vmadm, 2, 0, 1, 0));
                for (int i = 0; i < 25; i++) a.Word(Vector(Vmadl, 2, 0, 1, 0));
                return Thirds(a).Word(Sqv(2, 0, 0x10, 0)).Break();
            });

            Run(bus);

            Assert.Equal(0, ReadElements(bus, 0x100)[0]);
            Assert.Equal(0x8000, ReadElements(bus, 0x110)[0]);
            Assert.Equal(0x0000, ReadElements(bus, 0x120)[0]);
            Assert.Equal(0x3FEF, ReadElements(bus, 0x130)[0]);
        }

        // Addition consumes the carry, clamps what it writes, and keeps the unclamped sum's low half - see §8.
        [Fact]
        public void Addition_consumes_the_carry_and_keeps_the_unclamped_sum_in_the_accumulator()
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0x7FFF, 0x8000, 0x0001, 0xFFFF, 0, 0, 0, 0);
            Elements(bus, 0x010, 0x7FFF, 0x8000, 0x0001, 0x0001, 0, 0, 0, 0);

            Load(bus, a => Thirds(Li(a, 1, 0x0F0F)
                .Word(Cop2(6, 1, 0))
                .Word(Lqv(0, 0, 0, 0))
                .Word(Lqv(1, 0, 1, 0))
                .Word(Vector(Vadd, 2, 0, 1, 0)))
                .Word(Sqv(2, 0, 0x10, 0))
                .Word(Cop2(2, 16, 0))
                .Sw(16, 0, 0x200)
                .Break());

            Run(bus);

            Assert.Equal(new ushort[] { 0x7FFF, 0x8000, 3, 1, 0, 0, 0, 0 }, ReadElements(bus, 0x100));
            Assert.Equal(new ushort[] { 0xFFFF, 0x0001, 3, 1, 0, 0, 0, 0 }, ReadElements(bus, 0x130));
            Assert.Equal(0u, bus.Read32(MemoryMap.SpDmemBase + 0x200));
        }

        // The corpus's VCH vectors, worked through its own model element by element - see §8.
        [Fact]
        public void The_high_clip_sets_every_flag_from_the_signs_and_the_sum()
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0x0000, 0x0001, 0x7FFE, 0x7FFF, 0x8000, 0xFFFE, 0xFFFF, 0x0000);
            Elements(bus, 0x010, 0x8000, 0xFFFE, 0xFFFF, 0x0000, 0x0000, 0x0001, 0x7FFE, 0x7FFF);

            Load(bus, a => a
                .Word(Lqv(0, 0, 0, 0))
                .Word(Lqv(1, 0, 1, 0))
                .Word(Vector(Vch, 2, 0, 1, 0))
                .Word(Sqv(2, 0, 0x10, 0))
                .Word(Cop2(2, 16, 0)).Word(Cop2(2, 17, 1)).Word(Cop2(2, 18, 2))
                .Sw(16, 0, 0x200).Sw(17, 0, 0x204).Sw(18, 0, 0x208)
                .Break());

            Run(bus);

            Assert.Equal(new ushort[] { 0, 0xFFFF, 0xFFFF, 0, 0x8000, 2, 0x7FFE, 0 }, ReadElements(bus, 0x100));
            Assert.Equal(0xFFFF_DD77u, bus.Read32(MemoryMap.SpDmemBase + 0x200));
            Assert.Equal(0xFFFF_F033u, bus.Read32(MemoryMap.SpDmemBase + 0x204));
            Assert.Equal(0x0000_0022u, bus.Read32(MemoryMap.SpDmemBase + 0x208));
        }

        // The corpus's VRNDP tables: whether vs is odd decides the shift, and only a non-negative accumulator rounds - see §9.
        [Theory]
        [InlineData(4, new ushort[] { 0, 1, 0xFFFF, 0x8001, 1, 0x7FFF, 0x7FFF, 0x8000 }, new ushort[] { 0, 0, 0xFFFF, 0xFFFF, 0, 0x3FFF, 0x1FFF, 0xC000 }, new ushort[] { 0, 1, 0xFFFF, 0x8001, 1, 0, 0x4000, 0x8000 }, new ushort[] { 0, 1, 0, 0x7FFE, 0xFFFD, 0xBFFF, 0xA000, 0x3FFF })]
        [InlineData(5, new ushort[] { 0, 2, 0xFFFF, 0x8001, 0, 0x7FFF, 0x7FFF, 0x8000 }, new ushort[] { 0, 0, 0xFFFF, 0xFFFF, 0, 0x3FFE, 0x1FFE, 0xC000 }, new ushort[] { 0, 2, 0xFFFF, 0x8001, 0, 0x8001, 0xC002, 0x8000 }, new ushort[] { 0, 0, 0, 0x7FFE, 0xFFFE, 0x3FFF, 0x1FFF, 0x3FFF })]
        public void Rounding_adds_vt_shifted_by_the_parity_of_vs_only_to_a_non_negative_accumulator(
            int vs, ushort[] result, ushort[] top, ushort[] middle, ushort[] bottom)
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0x0000, 0x0001, 0x0001, 0x7FFF, 0xFFFF, 0x7FFF, 0x3FFF, 0x8000);
            Elements(bus, 0x010, 0x0000, 0x0001, 0xFFFF, 0xFFFF, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF);
            Elements(bus, 0x020, 0x0000, 0x0001, 0x0002, 0x7FFF, 0xFFFF, 0x8000, 0x8001, 0x8002);
            Elements(bus, 0x030, 0x1234, 0xFEDC, 0x0F0F, 0xF0F0, 0x5555, 0xAAAA, 0x0001, 0x8000);

            Load(bus, a => Thirds(a
                .Word(Lqv(0, 0, 0, 0))
                .Word(Lqv(1, 0, 1, 0))
                .Word(Vector(Vmudh, 2, 0, 1, 0))
                .Word(Vector(Vmadl, 2, 0, 1, 0))
                .Word(Lqv(0, 0, 2, 0))
                .Word(Lqv(vs, 0, 3, 0))
                .Word(Vector(Vrndp, 2, 0, vs, 0)))
                .Word(Sqv(2, 0, 0x10, 0))
                .Break());

            Run(bus);

            Assert.Equal(result, ReadElements(bus, 0x100));
            Assert.Equal(top, ReadElements(bus, 0x110));
            Assert.Equal(middle, ReadElements(bus, 0x120));
            Assert.Equal(bottom, ReadElements(bus, 0x130));
        }

        // A high half loads a hidden input the next low half consumes, and consuming clears it - see §10.
        [Theory]
        [InlineData(Vrcph, Vrcpl, 0xFFFA, 0x9E1B)]
        [InlineData(Vrsqh, Vrsql, 0x5BC2, 0xC2FF)]
        [InlineData(Vrcph, Vrsql, 0x5BC2, 0xC2FF)]
        [InlineData(Vrsqh, Vrcpl, 0xFFFA, 0x9E1B)]
        public void A_high_half_loads_an_input_that_one_low_half_consumes(int high, int low, int loaded, int cleared)
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0xE834, 0xE834, 0xE834, 0xE834, 0xE834, 0xE834, 0xE834, 0xE834);

            Load(bus, a => a
                .Word(Lqv(0, 0, 0, 0))
                .Word(Vector(high, 31, 0, 0, SelectorElement0))
                .Word(Vector(low, 2, 0, 0, SelectorElement0))
                .Word(Vector(low, 3, 0, 0, SelectorElement0))
                .Word(Sqv(2, 0, 0x10, 0))
                .Word(Sqv(3, 0, 0x11, 0))
                .Break());

            Run(bus);

            Assert.Equal((ushort)loaded, ReadElements(bus, 0x100)[0]);
            Assert.Equal((ushort)cleared, ReadElements(bus, 0x110)[0]);
        }

        // Either high half reads back the upper word of whichever reciprocal ran last - see §10.
        [Theory]
        [InlineData(Vrcp, 0xFFFA)]
        [InlineData(Vrsq, 0xFE5B)]
        public void A_high_half_reads_the_upper_word_of_the_last_reciprocal(int reciprocal, int expected)
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0xE834, 0xE834, 0xE834, 0xE834, 0xE834, 0xE834, 0xE834, 0xE834);

            Load(bus, a => a
                .Word(Lqv(0, 0, 0, 0))
                .Word(Vector(reciprocal, 1, 0, 1, SelectorElement0))
                .Word(Vector(Vrcph, 2, 0, 0, SelectorElement0))
                .Word(Vector(Vrsqh, 3, 0, 0, SelectorElement0))
                .Word(Sqv(2, 0, 0x10, 0))
                .Word(Sqv(3, 0, 0x11, 0))
                .Break());

            Run(bus);

            Assert.Equal((ushort)expected, ReadElements(bus, 0x100)[0]);
            Assert.Equal((ushort)expected, ReadElements(bus, 0x110)[0]);
        }

        // The accumulator has three readable thirds and every other selector reads zero - see §6.
        [Theory]
        [InlineData(0)]
        [InlineData(7)]
        [InlineData(11)]
        [InlineData(15)]
        public void Reading_the_accumulator_with_any_other_selector_reads_zero(int selector)
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF);

            Load(bus, a => a
                .Word(Lqv(0, 0, 0, 0))
                .Word(Lqv(4, 0, 0, 0))
                .Word(Sqv(4, 0, 0x20, 0))
                .Word(Vector(Vmulf, 3, 0, 0, 0))
                .Word(Vector(Vsar, 4, 0, 0, selector))
                .Word(Sqv(4, 0, 0x10, 0))
                .Break());

            Run(bus);

            Assert.Equal(0x7FFF, ReadElements(bus, 0x200)[0]);
            Assert.Equal(new ushort[8], ReadElements(bus, 0x100));
        }

        // An encoding without a documented operation zeroes its destination and still writes the accumulator - see §11.
        [Fact]
        public void An_undocumented_function_zeroes_its_destination_and_sums_into_the_accumulator()
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0xFFFF);
            Elements(bus, 0x010, 0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0xFFFF);
            Elements(bus, 0x020, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE);

            Load(bus, a => Thirds(a
                .Word(Lqv(0, 0, 0, 0))
                .Word(Lqv(1, 0, 1, 0))
                .Word(Lqv(2, 0, 2, 0))
                .Word(Vector(Vsut, 2, 0, 1, 0)))
                .Word(Sqv(2, 0, 0x10, 0))
                .Break());

            Run(bus);

            Assert.Equal(new ushort[8], ReadElements(bus, 0x100));
            Assert.Equal(new ushort[] { 0, 3, 0x800F, 0x7FFE, 0x7FFF, 0x7FFE, 0x7FFD, 0xFFFE }, ReadElements(bus, 0x130));
        }

        // The two no-operations touch nothing, not even the accumulator's low third - see §11.
        [Fact]
        public void The_vector_no_operation_changes_neither_its_destination_nor_the_accumulator()
        {
            var bus = new MarsBus();
            Elements(bus, 0x000, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF);
            Elements(bus, 0x020, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE);

            Load(bus, a => Thirds(a
                .Word(Lqv(0, 0, 0, 0))
                .Word(Vector(Vmulf, 3, 0, 0, 0))
                .Word(Lqv(2, 0, 2, 0))
                .Word(Vector(Vnop, 2, 0, 0, 0)))
                .Word(Sqv(2, 0, 0x10, 0))
                .Break());

            Run(bus);

            Assert.Equal(new ushort[] { 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE, 0xEEEE }, ReadElements(bus, 0x100));
            Assert.Equal(new ushort[] { 0x8002, 0x8002, 0x8002, 0x8002, 0x8002, 0x8002, 0x8002, 0x8002 }, ReadElements(bus, 0x130));
        }

        // The three thirds of the accumulator into V5, V6 and V7, then out to 0x110, 0x120 and 0x130.
        private static MipsAssembler Thirds(MipsAssembler a) => a
            .Word(Vector(Vsar, 5, 0, 0, 8))
            .Word(Vector(Vsar, 6, 0, 0, 9))
            .Word(Vector(Vsar, 7, 0, 0, 10))
            .Word(Sqv(5, 0, 0x11, 0))
            .Word(Sqv(6, 0, 0x12, 0))
            .Word(Sqv(7, 0, 0x13, 0));

        private static uint Vector(int function, int vd, int vt, int vs, int selector) =>
            (0x12u << 26) | (1u << 25) | ((uint)selector << 21) | ((uint)vt << 16) | ((uint)vs << 11) | ((uint)vd << 6) | (uint)function;

        private static uint Cop2(int operation, int rt, int register, int element = 0) =>
            (0x12u << 26) | ((uint)operation << 21) | ((uint)rt << 16) | ((uint)register << 11) | ((uint)element << 7);

        private static uint Transfer(uint opcode, int format, int vt, int element, int offset, int baseRegister) =>
            (opcode << 26) | ((uint)baseRegister << 21) | ((uint)vt << 16) | ((uint)format << 11) | ((uint)element << 7) | ((uint)offset & 0x7F);

        private static uint Lqv(int vt, int element, int offset, int baseRegister) => Transfer(0x32, Quad, vt, element, offset, baseRegister);

        private static uint Sqv(int vt, int element, int offset, int baseRegister) => Transfer(0x3A, Quad, vt, element, offset, baseRegister);

        private static MipsAssembler Li(MipsAssembler a, int rt, uint value) =>
            a.Lui(rt, (ushort)(value >> 16)).Ori(rt, rt, (ushort)value);

        private static void Elements(MarsBus bus, uint at, params ushort[] elements)
        {
            for (int i = 0; i < elements.Length; i++)
            {
                bus.SpDmem[at + i * 2] = (byte)(elements[i] >> 8);
                bus.SpDmem[at + i * 2 + 1] = (byte)elements[i];
            }
        }

        private static ushort[] ReadElements(MarsBus bus, uint at)
        {
            var elements = new ushort[8];
            for (int i = 0; i < 8; i++) elements[i] = (ushort)((bus.SpDmem[at + i * 2] << 8) | bus.SpDmem[at + i * 2 + 1]);

            return elements;
        }

        private static byte[] Bytes(MarsBus bus, uint at, int count) => bus.SpDmem.AsSpan((int)at, count).ToArray();

        // Each of the first 256 bytes holds its own offset, the corpus's pattern for telling bytes apart.
        private static void Counting(MarsBus bus)
        {
            for (int i = 0; i < 0x100; i++) bus.SpDmem[i] = (byte)i;
        }

        private static void Filled(MarsBus bus, uint at, byte value) => bus.SpDmem.AsSpan((int)at, 16).Fill(value);

        private static void Load(MarsBus bus, Func<MipsAssembler, MipsAssembler> program)
        {
            uint[] words = program(new MipsAssembler()).ToArray();
            for (int i = 0; i < words.Length; i++) bus.Write32(MemoryMap.SpImemBase + (uint)(i * 4), words[i]);
        }

        private static MarsBus Loaded(Func<MipsAssembler, MipsAssembler> program)
        {
            var bus = new MarsBus();

            Load(bus, program);
            return bus;
        }

        private static void Run(MarsBus bus)
        {
            bus.Write32(MemoryMap.SpPcBase, 0);
            bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, 0x01);
            bus.Tick(10_000);
        }
    }
}
