using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Cores
{
    // The half line the signal is on and the interrupt it raises, with no reference to compare against - see Mars_VideoTiming.md §4.
    public class MarsViTimingTests
    {
        private const uint Control = 0x00, Interrupt = 0x0C, CurrentLine = 0x10, VerticalSync = 0x18, HorizontalSync = 0x1C;

        private const uint NtscSync = 525, NtscLine = 3093, PalSync = 625, PalLine = 3177;

        // One half line at the NTSC defaults, rounded down: 3093 interface cycles a line against 93.75 MHz.
        private const long NtscHalfLine = 2979;

        private static MemoryBus Programmed(uint sync = NtscSync, uint line = NtscLine, uint interrupt = 0x3FF, bool serrate = false)
        {
            var bus = new MemoryBus();
            bus.Write32(MemoryMap.ViBase + Control, serrate ? 1u << 6 : 0);
            bus.Write32(MemoryMap.ViBase + VerticalSync, sync);
            bus.Write32(MemoryMap.ViBase + HorizontalSync, line);
            bus.Write32(MemoryMap.ViBase + Interrupt, interrupt);
            bus.Mi.Clear(MiInterrupt.VideoInterface);
            return bus;
        }

        private static uint Current(MemoryBus bus) => bus.Read32(MemoryMap.ViBase + CurrentLine);

        [Fact]
        public void The_half_line_advances_at_the_rate_the_registers_ask_for()
        {
            MemoryBus bus = Programmed();

            bus.Tick(NtscHalfLine * 2);
            Assert.Equal(2u, Current(bus));

            bus.Tick(NtscHalfLine * 8);
            Assert.Equal(10u, Current(bus));
        }

        // A line is longer in PAL and its interface clock faster, which very nearly cancel - see §1.1.
        [Fact]
        public void A_pal_signal_counts_at_its_own_rate()
        {
            MemoryBus ntsc = Programmed();
            MemoryBus pal = Programmed(PalSync, PalLine);

            ntsc.Tick(1_000_000);
            pal.Tick(1_000_000);

            Assert.Equal(334u, Current(ntsc));
            Assert.Equal(332u, Current(pal));
        }

        // The register's low bit is the field, so a progressive signal never sets it however many half lines pass - see §1.2.
        [Fact]
        public void A_progressive_signal_never_reports_a_field()
        {
            MemoryBus bus = Programmed();

            for (int i = 0; i < 99; i++)
            {
                bus.Tick(NtscHalfLine);
                Assert.Equal(0u, Current(bus) & 1);
            }

            Assert.Equal(98u, Current(bus));
        }

        [Fact]
        public void An_interlaced_signal_changes_field_when_the_count_wraps()
        {
            MemoryBus bus = Programmed(serrate: true);

            bus.Tick(NtscHalfLine * (NtscSync - 1));
            Assert.Equal(0u, Current(bus) & 1);

            bus.Tick(NtscHalfLine);
            Assert.Equal(1u, Current(bus) & 1);
            Assert.Equal(1u, Current(bus));

            bus.Tick(NtscHalfLine * NtscSync);
            Assert.Equal(0u, Current(bus) & 1);
        }

        [Fact]
        public void The_interrupt_is_raised_when_the_half_line_reaches_its_register()
        {
            MemoryBus bus = Programmed(interrupt: 4);

            bus.Tick(NtscHalfLine * 3);
            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));

            bus.Tick(NtscHalfLine);
            Assert.True(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));
        }

        // The comparison takes the low bit off the half line, so a register that names an odd one is never reached - see §2.
        [Fact]
        public void An_odd_interrupt_register_is_never_reached()
        {
            MemoryBus bus = Programmed(interrupt: 5);

            bus.Tick(NtscHalfLine * NtscSync * 2);
            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));
        }

        [Fact]
        public void Writing_the_current_line_clears_the_interrupt()
        {
            MemoryBus bus = Programmed(interrupt: 2);

            bus.Tick(NtscHalfLine * 2);
            Assert.True(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));

            bus.Write32(MemoryMap.ViBase + CurrentLine, 0);
            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));
        }

        // Nothing else clears it, which is what makes the handler's write the only way out - see §2.
        [Fact]
        public void Another_register_does_not_clear_the_interrupt()
        {
            MemoryBus bus = Programmed(interrupt: 2);

            bus.Tick(NtscHalfLine * 2);
            bus.Write32(MemoryMap.ViBase + Interrupt, 2);

            Assert.True(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));
        }

        [Fact]
        public void The_count_wraps_at_the_vertical_sync()
        {
            MemoryBus bus = Programmed();

            // The register reports the half line with its low bit taken off, so an odd count reads one lower - see §1.2.
            bus.Tick(NtscHalfLine * (NtscSync - 2));
            Assert.Equal(NtscSync - 3, Current(bus));

            bus.Tick(NtscHalfLine * 2);
            Assert.Equal(0u, Current(bus));
        }

        // An interface no game has programmed has no line rate at all, and must not divide by one - see §1.1.
        [Fact]
        public void An_unprogrammed_interface_does_not_count()
        {
            var bus = new MemoryBus();
            bus.Write32(MemoryMap.ViBase + Interrupt, 0);

            bus.Tick(10_000_000);

            Assert.Equal(0u, Current(bus));
            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));
        }

        // The vertical sync is the count's own bound, and a signal that has a line but no sync has nowhere to count to - see §1.2.
        [Fact]
        public void A_signal_with_a_line_but_no_vertical_sync_does_not_count()
        {
            MemoryBus bus = Programmed(sync: 0, interrupt: 0);

            bus.Tick(NtscHalfLine * 100);

            Assert.Equal(0u, Current(bus));
            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));
        }

        // The register is ten bits, and a picture has more half lines than nine bits can name - see §2.
        [Fact]
        public void An_interrupt_register_above_nine_bits_is_still_reached()
        {
            MemoryBus bus = Programmed(interrupt: 0x208);

            bus.Tick(NtscHalfLine * 0x207);
            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));

            bus.Tick(NtscHalfLine);
            Assert.True(bus.Mi.Pending.HasFlag(MiInterrupt.VideoInterface));
        }

        [Fact]
        public void A_line_of_no_length_does_not_count()
        {
            MemoryBus bus = Programmed(line: 0);

            bus.Tick(10_000_000);

            Assert.Equal(0u, Current(bus));
        }
    }
}
