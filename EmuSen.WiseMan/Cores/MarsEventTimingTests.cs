using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Cores
{
    // The VI and AI act only when due; each fact here is one a device stepped every tick kept, stated as its formula - see Mars_Performance.md §9.
    public class MarsEventTimingTests
    {
        private const long ProcessorClock = 93_750_000;
        private const long NtscClock = 48_681_818, PalClock = 49_656_530;
        private const uint Vi = MemoryMap.ViBase, Ai = MemoryMap.AiBase;
        private const uint CurrentLine = 0x10, VerticalSync = 0x18, HorizontalSync = 0x1C;
        private const uint NtscSync = 525, PalSync = 625, Line = 3093;
        private const long Threshold = Line * ProcessorClock;
        private const uint Rate = 1520;
        private const long Period = (Rate + 1) * ProcessorClock;
        private const uint Buffer = 0x0010_0000;

        private static MemoryBus Signal(uint sync = NtscSync)
        {
            var bus = new MemoryBus();
            bus.Write32(Vi + VerticalSync, sync);
            bus.Write32(Vi + HorizontalSync, Line);
            return bus;
        }

        private static void Playing(MemoryBus bus, uint length)
        {
            bus.Write32(Ai + AiInterface.DacRate, Rate);
            bus.Write32(Ai + AiInterface.Control, 1);
            Queue(bus, length);
        }

        private static void Queue(MemoryBus bus, uint length)
        {
            bus.Write32(Ai + AiInterface.DramAddress, Buffer);
            bus.Write32(Ai + AiInterface.Length, length);
        }

        private static uint Current(MemoryBus bus) => bus.Read32(Vi + CurrentLine);

        private static long Ceiling(long owed, long rate) => (owed + rate - 1) / rate;

        // The register shows every other half line, so it changes once a line.
        private static long CyclesToNextLine(MemoryBus bus)
        {
            uint start = Current(bus);
            for (long n = 1; n <= 20_000; n++)
            {
                bus.Tick(1);
                if (Current(bus) != start) return n;
            }

            return -1;
        }

        private static long CyclesToNextSample(MemoryBus bus)
        {
            long start = bus.Ai.SamplesPlayed;
            for (long n = 1; n <= 20_000; n++)
            {
                bus.Tick(1);
                if (bus.Ai.SamplesPlayed != start) return n;
            }

            return -1;
        }

        // Every change either device makes over <cycles> one-cycle ticks, by the cycle it came on.
        private static List<string> Trace(MemoryBus bus, int cycles)
        {
            var seen = new List<string>();
            uint line = Current(bus);
            long samples = bus.Ai.SamplesPlayed;

            for (int n = 1; n <= cycles; n++)
            {
                bus.Tick(1);
                if (Current(bus) == line && bus.Ai.SamplesPlayed == samples) continue;

                line = Current(bus);
                samples = bus.Ai.SamplesPlayed;
                seen.Add($"{n}: line {line}, samples {samples}, pending {bus.Mi.Pending}");
            }

            return seen;
        }

        private static byte[] State(MemoryBus bus)
        {
            using var stream = new MemoryStream();
            using (var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true)) bus.WriteState(w);
            return stream.ToArray();
        }

        private static void Load(MemoryBus bus, byte[] state)
        {
            using var r = new BinaryReader(new MemoryStream(state));
            bus.ReadState(r);
        }

        // Ticks of one cycle land on the due cycle itself, which is where a line that is owed has to turn.
        [Fact]
        public void A_line_turns_on_the_first_tick_whose_cycles_cover_it()
        {
            MemoryBus bus = Signal();

            Assert.Equal(Ceiling(2 * Threshold, 2 * NtscClock), CyclesToNextLine(bus));
            Assert.Equal(Ceiling(4 * Threshold, 2 * NtscClock) - Ceiling(2 * Threshold, 2 * NtscClock), CyclesToNextLine(bus));
        }

        // The cycles before the write were the NTSC clock's, and only the ones after it are PAL's.
        [Fact]
        public void A_switch_to_pal_mid_line_owes_the_cycles_before_it_at_the_ntsc_rate()
        {
            MemoryBus bus = Signal();
            bus.Tick(1000);

            bus.Write32(Vi + VerticalSync, PalSync);

            Assert.Equal(Ceiling(2 * Threshold - 1000 * 2 * NtscClock, 2 * PalClock), CyclesToNextLine(bus));
        }

        [Fact]
        public void A_signal_stopped_by_a_zero_sync_owes_nothing_for_the_time_it_stood()
        {
            MemoryBus bus = Signal();
            bus.Tick(1000);

            bus.Write32(Vi + VerticalSync, 0);
            bus.Tick(1_000_000);
            Assert.Equal(0u, Current(bus));

            bus.Write32(Vi + VerticalSync, NtscSync);
            Assert.Equal(Ceiling(2 * Threshold - 1000 * 2 * NtscClock, 2 * NtscClock), CyclesToNextLine(bus));
        }

        // Two samples and half a period more in one tick: the half period is left over when the buffer runs out.
        private static MemoryBus RanOutWithHalfAPeriodOwed()
        {
            var bus = new MemoryBus();
            Playing(bus, 8);
            bus.Tick(Ceiling(5 * Period / 2, NtscClock));
            Assert.Equal(2, bus.Ai.SamplesPlayed);
            return bus;
        }

        [Fact]
        public void A_buffer_queued_after_the_interface_stood_idle_starts_a_whole_period_later()
        {
            MemoryBus bus = RanOutWithHalfAPeriodOwed();
            bus.Tick(10_000);

            Queue(bus, 8);

            Assert.Equal(Ceiling(Period, NtscClock), CyclesToNextSample(bus));
        }

        // No tick has passed idle, so none has zeroed what the last one left over.
        [Fact]
        public void A_buffer_queued_on_the_cycle_the_last_one_ran_out_keeps_what_was_owed()
        {
            MemoryBus bus = RanOutWithHalfAPeriodOwed();
            long leftover = Ceiling(5 * Period / 2, NtscClock) * NtscClock - 2 * Period;

            Queue(bus, 8);

            Assert.Equal(Ceiling(Period - leftover, NtscClock), CyclesToNextSample(bus));
        }

        // One tick settles both devices at its end, and a thousand leave them owing since their last event; the state cannot tell.
        [Fact]
        public void A_state_is_the_same_bytes_however_the_ticks_that_reached_it_were_cut()
        {
            MemoryBus whole = Signal(), cut = Signal();
            Playing(whole, 0x1000);
            Playing(cut, 0x1000);

            whole.Tick(12_345);
            for (int n = 0; n < 12_345; n++) cut.Tick(1);

            Assert.Equal(State(whole), State(cut));
        }

        [Fact]
        public void A_state_loaded_into_a_fresh_bus_keeps_step_with_the_one_that_saved_it()
        {
            MemoryBus saver = Signal();
            Playing(saver, 0x1000);
            for (int n = 0; n < 12_345; n++) saver.Tick(1);

            byte[] state = State(saver);
            var loader = new MemoryBus();
            Load(loader, state);

            Assert.Equal(Trace(saver, 30_000), Trace(loader, 30_000));
        }

        // Its devices were due somewhere far later when the state came back, and must be due where the state says.
        [Fact]
        public void A_state_loaded_back_into_the_bus_that_saved_it_replays_what_followed()
        {
            MemoryBus bus = Signal();
            Playing(bus, 0x1000);
            bus.Tick(12_345);

            byte[] state = State(bus);
            List<string> first = Trace(bus, 30_000);
            bus.Tick(100_000);

            Load(bus, state);

            Assert.Equal(first, Trace(bus, 30_000));
        }
    }
}
