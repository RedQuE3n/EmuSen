using System;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Which security chip a cartridge carries, what IPL3 is handed for it, and the 6105's challenge - see Mars_Boot.md §6-§8.
    public class MarsCicTests
    {
        private const uint Dram = 0x0010_0000;

        // A boot code whose words sum to the total a reference lists, built from nothing but that total.
        private static byte[] ImageSummingTo(ulong total, char destination = 'E')
        {
            var ipl3 = new byte[RomImage.BootCodeLength];
            ulong full = total / 0xFFFF_FFFF;
            uint rest = (uint)(total % 0xFFFF_FFFF);

            for (int word = 0; word < (int)full; word++) ipl3.AsSpan(word * 4, 4).Fill(0xFF);
            ipl3[(int)full * 4] = (byte)(rest >> 24);
            ipl3[(int)full * 4 + 1] = (byte)(rest >> 16);
            ipl3[(int)full * 4 + 2] = (byte)(rest >> 8);
            ipl3[(int)full * 4 + 3] = (byte)rest;

            return SyntheticN64Rom.Build(destinationCode: destination, patches: (0, ipl3));
        }

        [Theory]
        [InlineData(0x0000_00D0_027F_DF31UL, CicChip.Nus6101)]
        [InlineData(0x0000_00CF_FB63_1223UL, CicChip.Nus6101)]
        [InlineData(0x0000_00D0_57C8_5244UL, CicChip.Nus6102)]
        [InlineData(0x0000_007C_5624_2373UL, CicChip.Nus6102)]
        [InlineData(0x0000_00D6_497E_414BUL, CicChip.Nus6103)]
        [InlineData(0x0000_011A_49F6_0E96UL, CicChip.Nus6105)]
        [InlineData(0x0000_00D6_D5BE_5580UL, CicChip.Nus6106)]
        public void Each_listed_boot_code_names_its_chip(ulong total, CicChip chip)
        {
            Assert.Equal(chip, RomImage.FromImage(ImageSummingTo(total)).CicChip);
        }

        [Fact]
        public void A_boot_code_nobody_lists_is_unknown_and_gets_the_6102_seed()
        {
            RomImage rom = RomImage.FromImage(SyntheticN64Rom.Build());

            Assert.Equal(CicChip.Unknown, rom.CicChip);
            Assert.Equal(0x3F, Cic.Seed(rom.CicChip));
        }

        [Theory]
        [InlineData(CicChip.Nus6101, 0x3F)]
        [InlineData(CicChip.Nus6102, 0x3F)]
        [InlineData(CicChip.Nus6103, 0x78)]
        [InlineData(CicChip.Nus6105, 0x91)]
        [InlineData(CicChip.Nus6106, 0x85)]
        public void Each_chip_hands_IPL3_its_own_seed(CicChip chip, byte seed)
        {
            Assert.Equal(seed, Cic.Seed(chip));
        }

        // The three things the 6105's IPL3 reads before writing: t3, ra and IMEM, plus the seed its checksum needs - see §6.
        [Fact]
        public void The_handoff_leaves_a_6105_what_IPL2_would_have()
        {
            var bus = new MemoryBus();
            var cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, RomImage.FromImage(ImageSummingTo(0x0000_011A_49F6_0E96UL, destination: 'P')));

            Assert.Equal(0x91UL, cpu.Gpr[22]);
            Assert.Equal(0xFFFF_FFFF_A400_0040UL, cpu.Gpr[11]);
            Assert.Equal(0xFFFF_FFFF_A400_1550UL, cpu.Gpr[31]);
            Assert.Equal(0UL, cpu.Gpr[20]);

            for (int i = 0; i < Boot.Ipl2Head.Length; i++)
            {
                Assert.Equal(Boot.Ipl2Head[i], bus.Read32(MemoryMap.SpDmemBase + 0x1000 + (uint)i * 4));
            }
        }

        // The last of IPL2's eight words is the first whose low twelve bits are zero, which is what ends the 6105's decryption loop - see §6.4.
        [Fact]
        public void The_eighth_word_of_IPL2_is_the_first_to_end_the_decryption_loop()
        {
            Assert.Equal(7, Array.FindIndex(Boot.Ipl2Head, word => (word & 0xFFF) == 0));
        }

        [Fact]
        public void A_6102_cartridge_still_gets_the_6102_seed()
        {
            var bus = new MemoryBus();
            var cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, RomImage.FromImage(ImageSummingTo(0x0000_00D0_57C8_5244UL)));

            Assert.Equal(0x3FUL, cpu.Gpr[22]);
            Assert.Equal(1UL, cpu.Gpr[20]);
        }

        // Pairs from Project64's pif2.dat, which recorded Banjo-Tooie's challenges and the answers they got - see §7.1.
        [Theory]
        [InlineData("000040001000040001000000000000", "BF9FD371C6EC62A8CBF9F9F9F9F9F9")]
        [InlineData("010040001000040001004000100000", "B4E62A8CBF9F937176ECA8C6371717")]
        [InlineData("810060001800060001004000100000", "3C6EA8C63F9F9D371C6E04E6371717")]
        [InlineData("AA006A001A00060001008000A00000", "D5553999EE6EC4E6E171F9F9171717")]
        [InlineData("2C004B001200040001000000C00000", "51115C6E117175555A8C6EC6A8C6EC")]
        [InlineData("33004C00130004000100C000300000", "A71751116D371BF9FE6E8C6EBF9F9F")]
        public void The_6105_gives_the_recorded_answer_to_a_recorded_challenge(string challenge, string response)
        {
            byte[] answer = Convert.FromHexString(challenge);

            Cic.Respond(answer, answer);

            Assert.Equal(response, Convert.ToHexString(answer));
        }

        // A block asking the challenge, in RDRAM where a game would build it, carried in through the serial interface.
        private static MemoryBus WithChallenge(CicChip chip, string challenge, byte command = SiInterface.ChallengeRequest)
        {
            var bus = new MemoryBus();
            ulong total = chip == CicChip.Nus6105 ? 0x0000_011A_49F6_0E96UL : 0x0000_00D0_57C8_5244UL;
            bus.Cart = RomImage.FromImage(ImageSummingTo(total));

            byte[] bytes = Convert.FromHexString(challenge);
            bus.Rdram[Dram + 0x2E] = 0xAA;
            bus.Rdram[Dram + 0x2F] = 0xBB;
            bytes.CopyTo(bus.Rdram, Dram + SiInterface.ChallengeAt);
            bus.Rdram[Dram + 0x3F] = command;

            bus.Write32(MemoryMap.SiBase + SiInterface.DramAddress, Dram);
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressWrite, 0);
            return bus;
        }

        // The answer reaches memory when the read's transfer is done - see Mars_Serial.md §2.3.
        private static void ReadBack(MemoryBus bus)
        {
            bus.Tick(SiInterface.TransferCycles);
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);
            bus.Tick(SiInterface.TransferCycles);
        }

        [Fact]
        public void A_6105_cartridge_answers_as_the_block_goes_out()
        {
            MemoryBus bus = WithChallenge(CicChip.Nus6105, "000040001000040001000000000000");

            Assert.Equal(SiInterface.ChallengeRequest, bus.PifRam[0x3F]);
            Assert.Equal(0x00, bus.PifRam[SiInterface.ChallengeAt]);

            ReadBack(bus);

            Assert.Equal("0000BF9FD371C6EC62A8CBF9F9F9F9F9F900", Convert.ToHexString(bus.Rdram, (int)Dram + 0x2E, 18));
        }

        [Fact]
        public void Any_other_chip_drops_the_request_and_leaves_the_challenge()
        {
            MemoryBus bus = WithChallenge(CicChip.Nus6102, "000040001000040001000000000000");

            ReadBack(bus);

            Assert.Equal("AABB00004000100004000100000000000000", Convert.ToHexString(bus.Rdram, (int)Dram + 0x2E, 18));
        }

        // The referee clears the one bit and keeps the rest of the byte, where Project64 answers only a byte of exactly 0x02 - see §7.2.
        [Fact]
        public void Answering_clears_the_request_bit_and_only_that_bit()
        {
            MemoryBus bus = WithChallenge(CicChip.Nus6105, "000040001000040001000000000000", command: 0x0A);

            ReadBack(bus);

            Assert.Equal(0xBF, bus.Rdram[Dram + SiInterface.ChallengeAt]);
            Assert.Equal(0x08, bus.Rdram[Dram + 0x3F]);
        }
    }
}
