using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Cores
{
    // The serial interface and the block the PIF runs for it - see Mars_Serial.md §4.
    public class MarsSerialTests
    {
        private const uint Dram = 0x0010_0000;

        private static MemoryBus WithBlock(params byte[] block)
        {
            var bus = new MemoryBus();
            for (uint i = 0; i < 64; i++) bus.Rdram[Dram + i] = i < block.Length ? block[i] : (byte)0;

            // The last byte asks the PIF to run the block, which is what a game sets before the transfer - see §2.
            bus.Rdram[Dram + 63] = 1;
            bus.Write32(MemoryMap.SiBase + SiInterface.DramAddress, Dram);
            return bus;
        }

        private static void Run(MemoryBus bus)
        {
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressWrite, 0);
        }

        [Fact]
        public void A_transfer_carries_sixty_four_bytes_each_way_and_interrupts()
        {
            MemoryBus bus = WithBlock(0xFE);

            Run(bus);
            Assert.True(bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));
            Assert.Equal(SiInterface.StatusInterrupt, bus.Read32(MemoryMap.SiBase + SiInterface.Status));

            bus.PifRam[7] = 0xA5;
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);
            Assert.Equal(0xA5, bus.Rdram[Dram + 7]);
        }

        [Fact]
        public void Writing_the_status_clears_the_interrupt()
        {
            MemoryBus bus = WithBlock(0xFE);

            Run(bus);
            bus.Write32(MemoryMap.SiBase + SiInterface.Status, 0);

            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));
            Assert.Equal(0u, bus.Read32(MemoryMap.SiBase + SiInterface.Status));
        }

        [Fact]
        public void A_block_runs_only_when_its_last_byte_asks()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Rdram[Dram + 63] = 0;

            Run(bus);

            Assert.Equal(0xFF, bus.PifRam[3]);
        }

        [Fact]
        public void An_info_command_names_a_controller_with_no_pak()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);

            Run(bus);

            Assert.Equal(0x03, bus.PifRam[1]);
            Assert.Equal(0x05, bus.PifRam[3]);
            Assert.Equal(0x00, bus.PifRam[4]);
            Assert.Equal(0x02, bus.PifRam[5]);
        }

        [Fact]
        public void A_controller_pak_shows_in_the_info_reply()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Si.Controllers[0].Pak = true;

            Run(bus);

            Assert.Equal(0x01, bus.PifRam[5]);
        }

        [Fact]
        public void A_state_command_reports_the_buttons_and_the_stick()
        {
            MemoryBus bus = WithBlock(0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Si.Controllers[0].Buttons = 0x8021;
            bus.Si.Controllers[0].StickX = 40;
            bus.Si.Controllers[0].StickY = -40;

            Run(bus);

            Assert.Equal(0x80, bus.PifRam[3]);
            Assert.Equal(0x21, bus.PifRam[4]);
            Assert.Equal(40, bus.PifRam[5]);
            Assert.Equal(0xD8, bus.PifRam[6]);
        }

        // Nothing in the port answers at all, which the length byte says rather than the reply - see §3.3.
        [Fact]
        public void An_empty_port_sets_the_no_reply_bit()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Si.Controllers[0].Present = false;

            Run(bus);

            Assert.Equal(Joybus.NoReply | 0x03, bus.PifRam[1]);
            Assert.Equal(0xFF, bus.PifRam[3]);
        }

        [Fact]
        public void A_reply_with_too_little_room_sets_the_over_run_bit()
        {
            MemoryBus bus = WithBlock(0x01, 0x02, 0x00, 0xFF, 0xFF, 0xFE);

            Run(bus);

            Assert.Equal(Joybus.OverRun | 0x02, bus.PifRam[1]);
            Assert.Equal(0x05, bus.PifRam[3]);
            Assert.Equal(0x00, bus.PifRam[4]);

            // The third byte of the reply had nowhere to go and must not have gone there anyway - see §3.3.
            Assert.Equal(Joybus.End, bus.PifRam[5]);
        }

        // A command the controller does not answer is a silence, not an error - see §3.2.
        [Fact]
        public void A_controller_pak_read_is_not_answered()
        {
            MemoryBus bus = WithBlock(0x03, 0x21, 0x02, 0x00, 0x00, 0xFE);

            Run(bus);

            Assert.Equal(Joybus.NoReply | 0x21, bus.PifRam[1]);
        }

        // The port that answers has to be one the first port is not, or the count cannot be seen - see §3.
        [Fact]
        public void A_zero_length_moves_to_the_next_channel()
        {
            MemoryBus bus = WithBlock(0x00, 0x00, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Si.Controllers[0].Present = false;
            bus.Si.Controllers[2].Present = true;

            Run(bus);

            Assert.Equal(0x03, bus.PifRam[3]);
            Assert.Equal(0x05, bus.PifRam[5]);
        }

        [Fact]
        public void A_command_moves_to_the_next_channel_as_well()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Si.Controllers[1].Present = false;

            Run(bus);

            Assert.Equal(0x03, bus.PifRam[1]);
            Assert.Equal(Joybus.NoReply | 0x03, bus.PifRam[7]);
        }

        // Both lengths are six bits, and a game that leaves anything above them must not be read as asking for more - see §3.
        [Fact]
        public void The_two_lengths_are_six_bits_each()
        {
            MemoryBus bus = WithBlock(0x41, 0x43, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);

            Run(bus);

            Assert.Equal(0x05, bus.PifRam[3]);
            Assert.Equal(0x02, bus.PifRam[5]);
        }

        // The end byte stops the walk before anything reads it as a length, which only the first entry can show - see §3.
        [Fact]
        public void The_end_byte_is_not_read_as_a_send_length()
        {
            MemoryBus bus = WithBlock(0xFE, 0x80, 0x00);

            Run(bus);

            Assert.Equal(0x80, bus.PifRam[1]);
        }

        [Fact]
        public void The_end_byte_is_not_read_as_a_receive_length()
        {
            MemoryBus bus = WithBlock(0x40, 0xFE, 0x00, 0x00, 0x00);

            Run(bus);

            Assert.Equal(0x00, bus.PifRam[2]);
            Assert.Equal(0x00, bus.PifRam[3]);
        }

        // A block only runs on its way in; on the way out the PIF is being read, not asked - see §2.
        [Fact]
        public void A_block_does_not_run_on_the_way_out()
        {
            var bus = new MemoryBus();
            byte[] block = { 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE };
            for (int i = 0; i < block.Length; i++) bus.PifRam[i] = block[i];
            bus.PifRam[63] = 1;

            bus.Write32(MemoryMap.SiBase + SiInterface.DramAddress, Dram);
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);

            Assert.Equal(0xFF, bus.PifRam[3]);
        }

        // Padding is not a channel, so the command after it still belongs to the first one - see §3.
        [Fact]
        public void Padding_does_not_move_to_the_next_channel()
        {
            MemoryBus bus = WithBlock(0xFF, 0xFD, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Si.Controllers[0].Present = true;

            Run(bus);

            Assert.Equal(0x05, bus.PifRam[5]);
        }

        [Fact]
        public void The_end_byte_stops_the_block()
        {
            MemoryBus bus = WithBlock(0xFE, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF);

            Run(bus);

            Assert.Equal(0xFF, bus.PifRam[4]);
        }

        [Fact]
        public void The_pif_clears_the_byte_it_was_started_by()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);

            Run(bus);

            Assert.Equal(0x00, bus.PifRam[63]);
        }
    }
}
