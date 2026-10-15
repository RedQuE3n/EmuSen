using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The referee's defect candidates, each proven on both engines before its fix - see Mercury_Native.md §13.
    public abstract class MercuryAuditClaims : IDisposable
    {
        private readonly List<MercuryRig> _rigs = new();

        protected abstract MercuryEngine Engine { get; }

        public void Dispose()
        {
            foreach (MercuryRig rig in _rigs) rig.Dispose();
        }

        private MercuryRig Load(byte cgbFlag = 0x00, GbModel model = GbModel.Auto, params (int Offset, byte[] Bytes)[] patches)
        {
            var rig = MercuryRig.Create(Engine, SyntheticGbRom.Build(cgbFlag: cgbFlag, patches: patches), model);
            _rigs.Add(rig);
            return rig;
        }

        private static IEnumerable<int> Range(int first, int last) => Enumerable.Range(first, last - first + 1);

        // Every model's gaps among the timer, interrupt and sound registers - see Mercury_Native.md §13.1.
        private static readonly int[] SharedGaps = new[] { 0xFF03 }.Concat(Range(0xFF08, 0xFF0E)).Concat(new[] { 0xFF15, 0xFF1F }).Concat(Range(0xFF27, 0xFF2F)).ToArray();

        private static void AssertUnmapped(MercuryRig rig, IEnumerable<int> addresses)
        {
            var wrong = new List<string>();
            foreach (int address in addresses)
            {
                foreach (byte value in new byte[] { 0x00, 0xFF, 0x5A })
                {
                    rig.Write(address, value);
                    byte read = rig.Read(address);
                    if (read != 0xFF) wrong.Add($"${address:X4} wrote {value:X2} read {read:X2}");
                }
            }
            Assert.True(wrong.Count == 0, string.Join("; ", wrong));
        }

        // A.1: mooneye unused_hwio-GS's list, and blargg 01-registers' $FF27-$FF2F row.
        [Fact]
        public void On_a_game_boy_every_unclaimed_io_address_reads_ff_and_keeps_no_write()
        {
            AssertUnmapped(Load(), SharedGaps.Concat(Range(0xFF4C, 0xFF7F)));
        }

        // A.1: in colour mode the gaps between the colour registers, and HDMA1-4, which Pan Docs calls write-only.
        [Fact]
        public void On_a_game_boy_color_in_colour_mode_the_gaps_and_the_hdma_addresses_read_ff()
        {
            var gaps = SharedGaps.Concat(new[] { 0xFF4C, 0xFF4E, 0xFF50 }).Concat(Range(0xFF51, 0xFF54)).Concat(Range(0xFF57, 0xFF67))
                .Concat(Range(0xFF6D, 0xFF6F)).Concat(new[] { 0xFF71 }).Concat(Range(0xFF78, 0xFF7F));
            AssertUnmapped(Load(cgbFlag: 0xC0), gaps);
        }

        // A.1: a Game Boy cartridge on the Color, as mooneye unused_hwio-C lists it.
        [Fact]
        public void A_game_boy_cartridge_on_the_color_sees_the_compatibility_modes_gaps_as_ff()
        {
            var gaps = SharedGaps.Concat(new[] { 0xFF4C, 0xFF4D, 0xFF4E, 0xFF69, 0xFF74 }).Concat(Range(0xFF50, 0xFF67))
                .Concat(Range(0xFF6B, 0xFF71)).Concat(Range(0xFF78, 0xFF7F));
            AssertUnmapped(Load(model: GbModel.GameBoyColor), gaps);
        }

        // A.1: $FF72-$FF75 as Pan Docs and unused_hwio-C have them; $FF74 only in colour mode.
        [Theory]
        [InlineData(true)]
        [InlineData(false)]
        public void The_colours_undocumented_registers_keep_their_writable_bits(bool colourMode)
        {
            MercuryRig rig = colourMode ? Load(cgbFlag: 0xC0) : Load(model: GbModel.GameBoyColor);
            foreach (byte value in new byte[] { 0x00, 0xFF, 0x5A })
            {
                rig.Write(0xFF72, value);
                rig.Write(0xFF73, (byte)~value);
                rig.Write(0xFF74, value);
                rig.Write(0xFF75, value);
                Assert.Equal(value, rig.Read(0xFF72));
                Assert.Equal((byte)~value, rig.Read(0xFF73));
                Assert.Equal(colourMode ? value : (byte)0xFF, rig.Read(0xFF74));
                Assert.Equal((byte)(0x8F | (value & 0x70)), rig.Read(0xFF75));
            }
        }

        // A.1: RP reads its unused bits and "no light received" as ones, after boot $3E - Pan Docs CGB_Registers.md, Power_Up_Sequence.md.
        [Fact]
        public void The_infrared_port_reads_its_writable_bits_and_no_light()
        {
            var rig = Load(cgbFlag: 0xC0);
            Assert.Equal(0x3E, rig.Read(0xFF56));
            rig.Write(0xFF56, 0xFF);
            Assert.Equal(0xFF, rig.Read(0xFF56));
            rig.Write(0xFF56, 0xC0);
            Assert.Equal(0xFE, rig.Read(0xFF56));
            rig.Write(0xFF56, 0x01);
            Assert.Equal(0x3F, rig.Read(0xFF56));
        }

        // A.1: OPRI's priority bit is read/write and the rest read one, in colour mode - Pan Docs CGB_Registers.md.
        [Fact]
        public void Opri_reads_back_its_one_bit_in_colour_mode()
        {
            var rig = Load(cgbFlag: 0xC0);
            rig.Write(0xFF6C, 0x01);
            Assert.Equal(0xFF, rig.Read(0xFF6C));
            rig.Write(0xFF6C, 0xFE);
            Assert.Equal(0xFE, rig.Read(0xFF6C));
        }

        // A.1: PCM12 and PCM34 are the channels' digital outputs on the Color, in either mode - Pan Docs Audio_details.md; unused_hwio-C reads them silent.
        [Theory]
        [InlineData(true)]
        [InlineData(false)]
        public void Pcm12_carries_pulse_twos_digital_output_in_its_high_nibble(bool colourMode)
        {
            MercuryRig rig = colourMode ? Load(cgbFlag: 0xC0) : Load(model: GbModel.GameBoyColor);
            Assert.Equal(0x00, rig.Read(0xFF76));
            Assert.Equal(0x00, rig.Read(0xFF77));

            rig.Write(0xFF26, 0x80);
            rig.Write(0xFF16, 0x80);        // NR21: 50% duty
            rig.Write(0xFF17, 0xF0);        // NR22: volume 15, DAC on
            rig.Write(0xFF18, 0x00);
            rig.Write(0xFF19, 0x87);        // NR24: trigger, a step every 1,024 cycles

            var seen = new HashSet<byte>();
            for (int i = 0; i < 64; i++)
            {
                rig.Tick(256);
                seen.Add(rig.Read(0xFF76));
                Assert.Equal(0x00, rig.Read(0xFF77));
            }
            Assert.Equal(new byte[] { 0x00, 0xF0 }, seen.OrderBy(b => b).ToArray());
        }

        // A, for mutant A7, which only the corpus caught: $FF46 reads back the page written.
        [Fact]
        public void The_dma_register_reads_back_the_page_written()
        {
            var rig = Load();
            rig.Write(0xFF46, 0xC1);
            Assert.Equal(0xC1, rig.Read(0xFF46));
        }

        // B: the timer at 262,144 Hz with TIMA one count from overflowing; DIV's write leaves the counter at zero, so the 16th cycle overflows.
        private MercuryRig TimerAboutToOverflow()
        {
            var rig = Load();
            rig.Write(0xFF0F, 0x00);
            rig.Write(0xFF07, 0x05);
            rig.Write(0xFF06, 0xFE);
            rig.Write(0xFF05, 0xFF);
            rig.Write(0xFF04, 0x00);
            return rig;
        }

        // B, Pan Docs Timer_Obscure_Behaviour.md rule 1: TIMA reads 0 for the machine cycle after it overflows, and a write then cancels the reload.
        [Fact]
        public void A_tima_write_in_the_cycle_after_overflow_cancels_the_reload_and_the_interrupt()
        {
            var rig = TimerAboutToOverflow();
            rig.Tick(16);
            Assert.Equal(0x00, rig.Read(0xFF05));
            rig.Write(0xFF05, 0x7F);
            rig.Tick(8);
            Assert.Equal(0x7F, rig.Read(0xFF05));
            Assert.Equal(0, rig.Read(0xFF0F) & 0x04);
        }

        // B, rule 2 and mooneye tima_write_reloading: a TIMA write in the cycle TMA is reloaded is lost to the reload.
        [Fact]
        public void A_tima_write_in_the_reload_cycle_is_ignored()
        {
            var rig = TimerAboutToOverflow();
            rig.Tick(16);
            rig.Tick(4);
            rig.Write(0xFF05, 0x7F);
            Assert.Equal(0xFE, rig.Read(0xFF05));
            Assert.Equal(0x04, rig.Read(0xFF0F) & 0x04);
        }

        // B, rule 3 and mooneye tma_write_reloading: a TMA write in the reload cycle reaches TIMA too.
        [Fact]
        public void A_tma_write_in_the_reload_cycle_reaches_tima_as_well()
        {
            var rig = TimerAboutToOverflow();
            rig.Tick(16);
            rig.Tick(4);
            rig.Write(0xFF06, 0x7F);
            Assert.Equal(0x7F, rig.Read(0xFF05));
            Assert.Equal(0x7F, rig.Read(0xFF06));
        }

        // B, for mutant B3: a DIV write that drops the selected bit overflows TIMA on a cycle's first T-cycle, and the reload's whole machine cycle still ignores a TIMA write.
        [Fact]
        public void A_reload_that_lands_early_in_its_cycle_still_ignores_a_tima_write_at_the_cycles_end()
        {
            var rig = TimerAboutToOverflow();
            rig.Tick(8);
            rig.Write(0xFF04, 0x00);
            rig.Tick(4);
            Assert.Equal(0x00, rig.Read(0xFF05));
            rig.Tick(4);
            rig.Write(0xFF05, 0x7F);
            Assert.Equal(0xFE, rig.Read(0xFF05));
        }

        // B: one machine cycle later both registers are the CPU's again.
        [Fact]
        public void After_the_reload_cycle_tima_and_tma_writes_are_ordinary()
        {
            var rig = TimerAboutToOverflow();
            rig.Tick(16);
            rig.Tick(4);
            rig.Tick(4);
            rig.Write(0xFF06, 0x33);
            rig.Write(0xFF05, 0x7F);
            Assert.Equal(0x7F, rig.Read(0xFF05));
            Assert.Equal(0x33, rig.Read(0xFF06));
        }

        // C: a dispatch from $02xx with IE and IF as given and SP as given; the step is the dispatch itself.
        private MercuryRig Dispatching(byte ie, byte iflags, ushort sp)
        {
            var rig = Load();
            rig.Write(0xFFFF, ie);
            rig.Write(0xFF0F, iflags);
            rig.Cpu.PC = 0x0234;
            rig.Cpu.SP = sp;
            rig.Cpu.Ime = true;
            rig.Step();
            return rig;
        }

        // C, mooneye ie_push round 1: the high byte of PC ($02) lands on IE and clears the timer's enable, so nothing is left to dispatch and PC becomes $0000.
        [Fact]
        public void A_push_that_clears_ie_before_the_vector_is_chosen_cancels_the_dispatch()
        {
            var rig = Dispatching(ie: 0x04, iflags: 0x04, sp: 0x0000);
            Assert.Equal(0x0000, rig.Cpu.PC);
            Assert.Equal(0x04, rig.Read(0xFF0F) & 0x1F);
            Assert.False(rig.Cpu.Ime);
            Assert.Equal(0x02, rig.Read(0xFFFF));
        }

        // C, ie_push round 4: the push leaves STAT enabled and not vblank, so STAT is dispatched and vblank stays requested.
        [Fact]
        public void A_push_that_changes_ie_before_the_vector_is_chosen_redirects_the_dispatch()
        {
            var rig = Dispatching(ie: 0x03, iflags: 0x03, sp: 0x0000);
            Assert.Equal(0x0048, rig.Cpu.PC);
            Assert.Equal(0x01, rig.Read(0xFF0F) & 0x1F);
        }

        // C, ie_push round 3: the low byte lands on IE after the vector was chosen, too late to cancel.
        [Fact]
        public void A_push_of_the_low_byte_onto_ie_is_too_late_to_cancel()
        {
            var rig = Dispatching(ie: 0x08, iflags: 0x08, sp: 0x0001);
            Assert.Equal(0x0058, rig.Cpu.PC);
            Assert.Equal(0x00, rig.Read(0xFF0F) & 0x1F);
            Assert.Equal(0x34, rig.Read(0xFFFF));
        }

        // C, GBCTR's EI: IME is set in the machine cycle after EI, so a second EI's effect is spent before the dispatch that follows it clears IME.
        [Fact]
        public void Ei_ei_before_a_dispatch_leaves_ime_clear_in_the_handler()
        {
            var rig = Load(patches: (0, new byte[] { 0xFB, 0xFB, 0x00 }));
            rig.Cpu.PC = SyntheticGbRom.EntryPoint;
            rig.Write(0xFFFF, 0x01);
            rig.Write(0xFF0F, 0x01);
            rig.Step();
            rig.Step();
            rig.Step();
            Assert.Equal(0x0040, rig.Cpu.PC);
            rig.Step();
            Assert.False(rig.Cpu.Ime);
        }

        // C, the referee's §2.14: STOP fetches two bytes, so its step reports the eight T-cycles it ran.
        [Fact]
        public void Stop_reports_the_cycles_of_its_two_fetches()
        {
            var rig = Load(patches: (0, new byte[] { 0x10, 0x00 }));
            rig.Cpu.PC = SyntheticGbRom.EntryPoint;
            long before = rig.Cpu.Cycles;
            Assert.Equal(8, rig.Step());
            Assert.Equal(before + 8, rig.Cpu.Cycles);
            Assert.Equal(SyntheticGbRom.EntryPoint + 2, rig.Cpu.PC);
        }

        // D: the LCD off, so only DMA closes OAM; OAM holds $12 and the pages at $C000 and $C100 hold $34 and $56.
        private MercuryRig DmaReady()
        {
            var rig = Load();
            rig.Write(0xFF40, 0x00);
            for (int i = 0; i < 0xA0; i++)
            {
                rig.Write(0xFE00 + i, 0x12);
                rig.Write(0xC000 + i, 0x34);
                rig.Write(0xC100 + i, 0x56);
            }
            return rig;
        }

        // D, mooneye oam_dma_start: "M = 1: nothing (OAM still accessible); M = 2: new DMA starts, OAM reads will return $FF".
        [Fact]
        public void Oam_stays_open_for_one_machine_cycle_after_the_dma_write()
        {
            var rig = DmaReady();
            rig.Write(0xFF46, 0xC0);
            rig.Tick(4);
            Assert.Equal(0x12, rig.Read(0xFE00));
            rig.Tick(4);
            Assert.Equal(0xFF, rig.Read(0xFE00));
        }

        // D, mooneye oam_dma_timing: a read 161 machine cycles after the write still sees $FF, one at 162 sees the copy.
        [Fact]
        public void Oam_is_closed_from_the_second_to_the_161st_machine_cycle_after_the_write()
        {
            var rig = DmaReady();
            rig.Write(0xFF46, 0xC0);
            rig.Tick(4 * 161);
            Assert.Equal(0xFF, rig.Read(0xFE00));
            rig.Tick(4);
            Assert.Equal(0x34, rig.Read(0xFE00));
            Assert.Equal(0x34, rig.Read(0xFE9F));
        }

        // D, mooneye oam_dma_restart: a second write keeps OAM closed while the first transfer runs on, and the second's own 160 cycles follow.
        [Fact]
        public void A_restarted_dma_keeps_oam_closed_and_ends_160_cycles_after_its_start()
        {
            var rig = DmaReady();
            rig.Write(0xFF46, 0xC0);
            rig.Tick(4 * 10);
            rig.Write(0xFF46, 0xC1);
            rig.Tick(4);
            Assert.Equal(0xFF, rig.Read(0xFE00));
            rig.Tick(4 * 160);
            Assert.Equal(0xFF, rig.Read(0xFE00));
            rig.Tick(4);
            Assert.Equal(0x56, rig.Read(0xFE00));
            Assert.Equal(0x56, rig.Read(0xFE9F));
        }

        // D: version 9 appends the start delay after the mixer; a version-8 state, which has none, loads with no start pending and the rest as it was.
        [Fact]
        public void A_version_8_state_loads_with_no_dma_start_pending()
        {
            var rig = DmaReady();
            rig.Write(0xFF46, 0xC0);
            byte[] v9 = rig.State();
            Assert.Equal(9, BitConverter.ToInt32(v9, 4));
            Assert.Equal(5, BitConverter.ToInt32(v9, v9.Length - 4));

            byte[] v8 = v9[..^4];
            BitConverter.GetBytes(8).CopyTo(v8, 4);
            rig.LoadState(new MemoryStream(v8));
            byte[] back = rig.State();
            Assert.Equal(0, BitConverter.ToInt32(back, back.Length - 4));
            Assert.True(back.AsSpan(8, back.Length - 12).SequenceEqual(v9.AsSpan(8, v9.Length - 12)));
        }

        // F, mooneye emulator-only/mbc1/ram_64kb and GBCTR's MBC1 RAM wiring: an 8 KiB chip has no pins for BANK2, so every RAM bank is the one bank.
        [Fact]
        public void An_mbc1_with_one_ram_bank_answers_every_bank_number_with_it()
        {
            var rig = MercuryRig.Create(Engine, SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x03, ramSizeCode: 0x02));
            _rigs.Add(rig);
            rig.Write(0x0000, 0x0A);
            rig.Write(0x6000, 0x01);
            rig.Write(0x4000, 0xFF);
            rig.Write(0xA123, 0x5A);
            foreach (byte bank in new byte[] { 0x00, 0x01, 0x02, 0x03 })
            {
                rig.Write(0x4000, bank);
                Assert.Equal(0x5A, rig.Read(0xA123));
            }
        }

        // F: a 1 MiB MBC1 image with each bank's number at offset $2000 of the bank, and bank $10 opening with bank 0's logo or not.
        private MercuryRig Mbc1Megabyte(bool logoInBank16, bool blankLogo = false)
        {
            byte[] image = SyntheticGbRom.Build(romBanks: 64, cartridgeType: 0x01);
            for (int i = 0; i < 48; i++) image[0x104 + i] = blankLogo ? (byte)0x00 : (byte)(0x31 + i * 7);
            if (logoInBank16) Array.Copy(image, 0x104, image, (0x10 * 0x4000) + 0x104, 48);
            for (int bank = 0; bank < 64; bank++) image[(bank * 0x4000) + 0x2000] = (byte)bank;
            var rig = MercuryRig.Create(Engine, image);
            _rigs.Add(rig);
            return rig;
        }

        // F, mooneye multicart_rom_8Mb and Pan Docs MBC1.md "MBC1M": BANK1 is four bits wide and BANK2 selects banks $00, $10, $20, $30.
        [Fact]
        public void An_mbc1_multicart_puts_bank2_on_bank_bits_4_and_5()
        {
            var rig = Mbc1Megabyte(logoInBank16: true);
            rig.Write(0x6000, 0x01);
            rig.Write(0x4000, 0x01);
            Assert.Equal(0x10, rig.Read(0x2000));
            rig.Write(0x2000, 0x01);
            Assert.Equal(0x11, rig.Read(0x6000));
            rig.Write(0x2000, 0x10);
            Assert.Equal(0x10, rig.Read(0x6000));
            rig.Write(0x4000, 0x00);
            Assert.Equal(0x00, rig.Read(0x6000));
            rig.Write(0x4000, 0x03);
            rig.Write(0x2000, 0x0F);
            Assert.Equal(0x3F, rig.Read(0x6000));
        }

        // F: without bank 0's logo in bank $10 the same image is an ordinary MBC1, BANK2 on bits 5 and 6.
        [Fact]
        public void A_megabyte_mbc1_without_the_second_logo_banks_as_an_ordinary_mbc1()
        {
            var rig = Mbc1Megabyte(logoInBank16: false);
            rig.Write(0x6000, 0x01);
            rig.Write(0x4000, 0x01);
            Assert.Equal(0x20, rig.Read(0x2000));
            rig.Write(0x2000, 0x01);
            Assert.Equal(0x21, rig.Read(0x6000));
        }

        // F: two blank logo areas are not the same logo, so a synthetic or blank image is never taken for a multicart.
        [Fact]
        public void A_megabyte_mbc1_with_a_blank_logo_in_both_banks_is_ordinary()
        {
            var rig = Mbc1Megabyte(logoInBank16: true, blankLogo: true);
            rig.Write(0x6000, 0x01);
            rig.Write(0x4000, 0x01);
            Assert.Equal(0x20, rig.Read(0x2000));
        }
    }

    public sealed class MercuryAuditTests : MercuryAuditClaims
    {
        protected override MercuryEngine Engine => MercuryEngine.Mercury;
    }

    public sealed class MercuryRtAuditTests : MercuryAuditClaims
    {
        protected override MercuryEngine Engine => MercuryEngine.MercuryRt;
    }
}
