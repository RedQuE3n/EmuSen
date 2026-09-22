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

        // What a game does: the block in, and then out, which is when the PIF runs it; the answer is in memory once the read is done - see §2 and §2.3.
        private static void Run(MemoryBus bus)
        {
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressWrite, 0);
            bus.Tick(SiInterface.TransferCycles);
            bus.Write32(MemoryMap.SiBase + SiInterface.Status, 0);
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);
        }

        private static void Finish(MemoryBus bus) => bus.Tick(SiInterface.TransferCycles);

        // The bytes move at once; the interface is busy and silent until the transfer's time has passed, and then it interrupts - see Mars_Serial.md §2.2.
        [Fact]
        public void A_transfer_carries_sixty_four_bytes_each_way_and_interrupts_when_its_time_has_passed()
        {
            MemoryBus bus = WithBlock(0xFE);

            Run(bus);
            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));
            Assert.Equal(SiInterface.StatusDmaBusy, bus.Read32(MemoryMap.SiBase + SiInterface.Status));

            bus.Tick(SiInterface.TransferCycles - 1);
            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));

            bus.Tick(1);
            Assert.True(bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));
            Assert.Equal(SiInterface.StatusInterrupt, bus.Read32(MemoryMap.SiBase + SiInterface.Status));

            bus.PifRam[7] = 0xA5;
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);
            Finish(bus);
            Assert.Equal(0xA5, bus.Rdram[Dram + 7]);
        }

        // Mischief Makers starts the read, packs its next block into the same buffer at once, and reads the buffer later: the reply has to land last - see Mars_Serial.md §2.3.
        [Fact]
        public void A_read_lands_in_memory_when_the_transfer_finishes_so_a_write_made_meanwhile_is_overwritten()
        {
            MemoryBus bus = WithBlock(0xFF, 0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Si.Controllers[0].Buttons = 0x1000;
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressWrite, 0);
            bus.Tick(SiInterface.TransferCycles);
            bus.Write32(MemoryMap.SiBase + SiInterface.Status, 0);

            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);
            for (uint i = 0; i < 64; i += 4) bus.Rdram[Dram + i + 3] = 0xFF;
            bus.Rdram[Dram + 4] = 0;

            bus.Tick(SiInterface.TransferCycles - 1);
            Assert.Equal(0x00, bus.Rdram[Dram + 4]);
            bus.Tick(1);

            Assert.Equal(0x10, bus.Rdram[Dram + 4]);
            Assert.Equal(0x04, bus.Rdram[Dram + 2]);
            Assert.True(bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));
        }

        [Fact]
        public void Writing_the_status_clears_the_interrupt()
        {
            MemoryBus bus = WithBlock(0xFE);

            Run(bus);
            bus.Tick(SiInterface.TransferCycles);
            Assert.True(bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));
            bus.Write32(MemoryMap.SiBase + SiInterface.Status, 0);

            Assert.False(bus.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));
            Assert.Equal(0u, bus.Read32(MemoryMap.SiBase + SiInterface.Status));
        }

        // A state written while a transfer is under way carries its interrupt, in a format no older state differs from - see §2.2.
        [Fact]
        public void A_state_saved_during_a_transfer_still_interrupts_when_it_is_loaded()
        {
            MemoryBus bus = WithBlock(0xFE);
            Run(bus);
            bus.Tick(100);
            bus.Rdram[Dram] = 0x00;

            using var stream = new System.IO.MemoryStream();
            using (var w = new System.IO.BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true)) bus.WriteState(w);

            var loaded = new MemoryBus();
            stream.Position = 0;
            using (var r = new System.IO.BinaryReader(stream, System.Text.Encoding.UTF8, leaveOpen: true)) loaded.ReadState(r);

            Assert.Equal(bus.Si.Due, loaded.Si.Due);
            loaded.Tick(SiInterface.TransferCycles - 101);
            Assert.False(loaded.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));
            loaded.Tick(1);
            Assert.True(loaded.Mi.Pending.HasFlag(MiInterrupt.SerialInterface));

            // The read under way was carried too, and its bytes landed with the interrupt - see Mars_Serial.md §2.3.
            Assert.Equal(0xFE, loaded.Rdram[Dram]);
            Assert.Equal(-1, loaded.Si.PendingRead);

            // Idle again, the three addresses are gone, and the state is the length it always was.
            using var idle = new System.IO.MemoryStream();
            using (var w = new System.IO.BinaryWriter(idle, System.Text.Encoding.UTF8, leaveOpen: true)) loaded.WriteState(w);
            Assert.Equal(stream.Length - 24, idle.Length);
        }

        // The referee's read walks the block whatever the last byte says; only the challenge bit steers it - see §2.
        [Fact]
        public void A_block_runs_on_a_read_whatever_its_last_byte_says()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);
            bus.Rdram[Dram + 63] = 0;

            Run(bus);

            Assert.Equal(0x05, bus.PifRam[3]);
        }

        // The regression the controller check found: a block written once and read every frame - see §2.
        [Fact]
        public void Each_read_runs_the_block_again_with_the_buttons_as_they_are_now()
        {
            MemoryBus bus = WithBlock(0xFF, 0x01, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0xFE);
            Run(bus);
            Finish(bus);
            Assert.Equal(0x00, bus.Rdram[Dram + 4]);

            bus.Si.Controllers[0].Buttons = 0x1000;
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);
            Finish(bus);

            Assert.Equal(new byte[] { 0x10, 0x00 }, bus.Rdram[(int)(Dram + 4)..(int)(Dram + 6)]);
        }

        [Fact]
        public void A_block_does_not_run_on_the_way_in()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);

            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressWrite, 0);

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
            bus.Si.Controllers[0].Pak = new ControllerPak(null);

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

        // The block already in PIF RAM runs as it is read out, and what goes to memory is the answer - see §2.
        [Fact]
        public void A_block_runs_on_the_way_out()
        {
            var bus = new MemoryBus();
            byte[] block = { 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE };
            for (int i = 0; i < block.Length; i++) bus.PifRam[i] = block[i];
            bus.PifRam[63] = 1;

            bus.Write32(MemoryMap.SiBase + SiInterface.DramAddress, Dram);
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);
            Finish(bus);

            Assert.Equal(0x05, bus.PifRam[3]);
            Assert.Equal(0x05, bus.Rdram[Dram + 3]);
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

        // The referee's walk leaves the byte alone, so a block written once is walked on every read - see §2.
        [Fact]
        public void The_pif_leaves_the_byte_it_was_started_by()
        {
            MemoryBus bus = WithBlock(0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE);

            Run(bus);

            Assert.Equal(0x01, bus.PifRam[63]);
        }
    }
}
