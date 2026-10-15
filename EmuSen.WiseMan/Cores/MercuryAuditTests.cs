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
