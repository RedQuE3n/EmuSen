using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Cores
{
    // The audio interface as the FPGA core builds it; no corpus group grades it, so these hold its rules - see Mars_Audio.md §5.
    public class MarsAudioTests
    {
        private const uint Ai = MemoryMap.AiBase;
        private const uint Buffer = 0x0010_0000;
        private const long ProcessorClock = 93_750_000;
        private const uint Rate = 1520;

        private static MemoryBus Playing(uint dacRate = Rate)
        {
            var bus = new MemoryBus();
            bus.Write32(Ai + AiInterface.DacRate, dacRate);
            bus.Write32(Ai + AiInterface.Control, 1);
            return bus;
        }

        private static void Queue(MemoryBus bus, uint address, uint length)
        {
            bus.Write32(Ai + AiInterface.DramAddress, address);
            bus.Write32(Ai + AiInterface.Length, length);
        }

        // Exactly the cycles <samples> DAC periods take, from a standing start.
        private static void PlaySamples(MemoryBus bus, long samples, uint dacRate = Rate)
        {
            long period = System.Math.Max(dacRate + 1, AiInterface.ShortestPeriod);
            long clock = bus.Vi.VideoClock;
            bus.Tick((samples * period * ProcessorClock + clock - 1) / clock);
        }

        private static uint Status(MemoryBus bus) => bus.Read32(Ai + AiInterface.Status);

        private static bool Interrupted(MemoryBus bus) => bus.Mi.Pending.HasFlag(MiInterrupt.AudioInterface);

        [Fact]
        public void The_status_reads_as_the_fpga_core_builds_it()
        {
            var bus = new MemoryBus();
            Assert.Equal(AiInterface.StatusAlwaysSet, Status(bus));

            bus.Write32(Ai + AiInterface.Control, 1);
            Queue(bus, Buffer, 0x100);
            Assert.Equal(0x4310_0000u, Status(bus));

            Queue(bus, Buffer + 0x100, 0x100);
            Assert.Equal(0xC310_0001u, Status(bus));
        }

        // The interrupt marks a buffer beginning, which for the first one is the moment its length is written - see §3.1.
        [Fact]
        public void A_length_written_while_idle_begins_a_buffer_and_raises_the_interrupt()
        {
            var bus = Playing();
            Assert.False(Interrupted(bus));

            Queue(bus, Buffer, 0x100);

            Assert.True(Interrupted(bus));
            Assert.Equal(0x100u, bus.Read32(Ai + AiInterface.Length));

            bus.Write32(Ai + AiInterface.Status, 0);
            Assert.False(Interrupted(bus));
        }

        [Fact]
        public void A_third_buffer_is_dropped_while_two_are_held()
        {
            var bus = Playing();
            Queue(bus, Buffer, 8);
            Queue(bus, Buffer + 8, 8);
            Queue(bus, Buffer + 16, 0x800);

            PlaySamples(bus, 4);

            Assert.Equal(0u, Status(bus) & AiInterface.StatusBusy);
            Assert.Equal(4, bus.Ai.SamplesPlayed);
        }

        [Fact]
        public void The_remaining_length_counts_down_as_samples_play()
        {
            var bus = Playing();
            Queue(bus, Buffer, 0x100);

            PlaySamples(bus, 10);

            Assert.Equal(0x100u - 40, bus.Read32(Ai + AiInterface.Length));
        }

        // The waiting buffer begins as the playing one ends, and raises the interrupt again; the last one ends silently.
        [Fact]
        public void A_finished_buffer_hands_over_to_the_waiting_one_with_an_interrupt()
        {
            var bus = Playing();
            Queue(bus, Buffer, 8);
            Queue(bus, Buffer + 0x40, 8);
            bus.Write32(Ai + AiInterface.Status, 0);

            PlaySamples(bus, 2);
            Assert.True(Interrupted(bus));
            Assert.Equal(AiInterface.StatusBusy, Status(bus) & (AiInterface.StatusBusy | AiInterface.StatusFull));

            bus.Write32(Ai + AiInterface.Status, 0);
            PlaySamples(bus, 2);

            Assert.False(Interrupted(bus));
            Assert.Equal(0u, Status(bus) & AiInterface.StatusBusy);
        }

        // Sixteen bits a side, big-endian, left first.
        [Fact]
        public void A_sample_is_two_big_endian_halves_left_first()
        {
            var bus = Playing();
            bus.Write32(Buffer, 0x1234_FEDC);
            bus.Write32(Buffer + 4, 0x8000_7FFF);
            Queue(bus, Buffer, 8);

            PlaySamples(bus, 2);

            Assert.Equal(new short[] { 0x1234, unchecked((short)0xFEDC), short.MinValue, short.MaxValue }, bus.Ai.Drain(int.MaxValue));
        }

        // One sample every DACRATE + 1 cycles of the video clock, counted off the processor's - see §2.
        [Fact]
        public void Samples_play_at_the_rate_the_dac_divides_the_video_clock_by()
        {
            var bus = Playing();
            Queue(bus, Buffer, 0x1000);

            long period = (Rate + 1) * ProcessorClock;
            long clock = bus.Vi.VideoClock;
            bus.Tick((100 * period + clock - 1) / clock - 1);
            Assert.Equal(99, bus.Ai.SamplesPlayed);

            bus.Tick(1);
            Assert.Equal(100, bus.Ai.SamplesPlayed);
            Assert.Equal((int)System.Math.Round(clock / (double)(Rate + 1)), bus.Ai.SampleRate);
        }

        // The RTL's floor is 512 video-clock cycles: a rate one short of it plays at it, and one past it does not.
        [Theory]
        [InlineData(0x010u, 512)]
        [InlineData(0x1FEu, 512)]
        [InlineData(0x1FFu, 512)]
        [InlineData(0x200u, 513)]
        public void A_period_shorter_than_the_floor_plays_at_the_floor(uint dacRate, int period)
        {
            var bus = Playing(dacRate);

            Assert.Equal((int)System.Math.Round(bus.Vi.VideoClock / (double)period), bus.Ai.SampleRate);
        }

        // The RTL keeps bits 23 to 3 of an address, so a buffer written part way into a doubleword plays from its start.
        [Fact]
        public void An_address_drops_its_low_three_bits()
        {
            var bus = Playing();
            bus.Write32(Buffer, 0x0102_0304);
            bus.Write32(Buffer + 4, 0x0506_0708);

            Queue(bus, Buffer + 4, 8);
            PlaySamples(bus, 1);

            Assert.Equal(new short[] { 0x0102, 0x0304 }, bus.Ai.Drain(int.MaxValue));
        }

        // And bits 17 to 3 of a length, so a buffer is whole doublewords and anything less is not played.
        [Fact]
        public void A_length_drops_its_low_three_bits()
        {
            var bus = Playing();
            Queue(bus, Buffer, 0x0F);

            Assert.Equal(8u, bus.Read32(Ai + AiInterface.Length));
            PlaySamples(bus, 3);
            Assert.Equal(2, bus.Ai.SamplesPlayed);
        }

        [Fact]
        public void Before_a_game_sets_a_rate_the_frontends_default_stands()
        {
            Assert.Equal(AiInterface.DefaultSampleRate, new MemoryBus().Ai.SampleRate);
        }

        [Fact]
        public void Nothing_moves_while_the_dma_is_off()
        {
            var bus = Playing();
            bus.Write32(Ai + AiInterface.Control, 0);
            Queue(bus, Buffer, 0x100);

            PlaySamples(bus, 10);

            Assert.Equal(0, bus.Ai.SamplesPlayed);
            Assert.Equal(0x100u, bus.Read32(Ai + AiInterface.Length));
        }

        // A buffer that ends on an 8KB page leaves its carry to the next one, which starts a page further on - see §3.2.
        [Fact]
        public void A_buffer_ending_on_a_page_starts_the_next_one_a_page_further_on()
        {
            var bus = Playing();
            bus.Write32(Buffer + AiInterface.PageSize, 0x1111_2222);
            bus.Write32(Buffer + 2 * AiInterface.PageSize, 0x3333_4444);

            Queue(bus, Buffer + AiInterface.PageSize - 8, 8);
            Queue(bus, Buffer + AiInterface.PageSize, 8);
            PlaySamples(bus, 3);

            short[] played = bus.Ai.Drain(int.MaxValue);
            Assert.Equal(new short[] { 0x3333, 0x4444 }, played[4..6]);
        }

        // Inside one buffer the late carry lands before the next fetch, so crossing a page is heard as nothing at all.
        [Fact]
        public void Inside_a_buffer_crossing_a_page_is_continuous()
        {
            var bus = Playing();
            uint page = Buffer + AiInterface.PageSize;
            bus.Write32(page - 8, 0x0101_0202);
            bus.Write32(page - 4, 0x0303_0404);
            bus.Write32(page, 0x0505_0606);
            bus.Write32(page + 4, 0x0707_0808);

            Queue(bus, page - 8, 16);
            PlaySamples(bus, 4);

            Assert.Equal(new short[] { 0x0101, 0x0202, 0x0303, 0x0404, 0x0505, 0x0606, 0x0707, 0x0808 }, bus.Ai.Drain(int.MaxValue));
        }

        [Fact]
        public void Draining_takes_whole_pairs_and_leaves_the_rest()
        {
            var bus = Playing();
            Queue(bus, Buffer, 16);
            PlaySamples(bus, 4);

            Assert.Equal(2, bus.Ai.Drain(1).Length);
            Assert.Equal(6, bus.Ai.Drain(int.MaxValue).Length);
            Assert.Empty(bus.Ai.Drain(int.MaxValue));
        }

        // Nothing draining must not grow memory without bound; the oldest pair goes first - see §4.
        [Fact]
        public void An_undrained_queue_keeps_only_the_newest_samples()
        {
            var bus = Playing(dacRate: 0);
            Queue(bus, Buffer, 0x3_FFF8);

            PlaySamples(bus, AiInterface.MaxBufferedSamples / 2 + 10, dacRate: 0);

            Assert.Equal(AiInterface.MaxBufferedSamples, bus.Ai.BufferedSamples);
            Assert.Equal(AiInterface.MaxBufferedSamples / 2 + 10, bus.Ai.SamplesPlayed);
        }
    }
}
