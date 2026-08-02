using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;
using EmuSen.WiseMan.Fixtures;
using Sa1Chip = EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1.Sa1;

namespace EmuSen.WiseMan.Coprocessors
{
    // Detection, the two-way message/interrupt channel, the reset handshake
    // that actually starts the second CPU, and the vector override - see
    // Venus_SA1.md §1 and §4.
    public class Sa1ControlTests
    {
        private const ushort Ccnt = 0x2200, Sie = 0x2201, Sic = 0x2202;
        private const ushort Crv = 0x2203, Cnv = 0x2205, Civ = 0x2207;
        private const ushort Scnt = 0x2209, Cie = 0x220A, Cic = 0x220B;
        private const ushort Snv = 0x220C, Siv = 0x220E;
        private const ushort Sfr = 0x2300, Cfr = 0x2301;

        private static Sa1Chip Chip() => new(new byte[0x100000], new byte[0x8000]);

        // --- Detection ---

        [Fact]
        public void Map_mode_23_builds_the_coprocessor()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildSa1());
            Assert.NotNull(core.Cart!.Sa1);
            Assert.Equal("SA-1", core.Cart.MapperName);
        }

        [Fact]
        public void An_ordinary_lorom_gets_no_coprocessor()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.Build());
            Assert.Null(core.Cart!.Sa1);
            Assert.Equal("LoROM", core.Cart.MapperName);
        }

        // --- Messages both ways ---

        [Fact]
        public void Message_from_the_scpu_appears_in_cfr()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Ccnt, 0x2A);
            Assert.Equal(0x0A, sa1.ReadRegister(Cfr) & 0x0F);
        }

        [Fact]
        public void Message_from_the_sa1_appears_in_sfr()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Scnt, 0x05);
            Assert.Equal(0x05, sa1.ReadRegister(Sfr) & 0x0F);
        }

        // --- Interrupt flags ---

        [Fact]
        public void Sa1_can_raise_and_the_scpu_can_clear_an_irq()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Sie, 0x80);
            sa1.WriteRegister(Scnt, 0x80);

            Assert.True(sa1.ScpuIrqPending);
            Assert.Equal(0x80, sa1.ReadRegister(Sfr) & 0x80);

            sa1.WriteRegister(Sic, 0x80);
            Assert.False(sa1.ScpuIrqPending);
        }

        [Fact]
        public void An_irq_the_scpu_has_not_enabled_stays_off_its_line()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Scnt, 0x80);

            // Still latched and visible in the status register, just not asserted.
            Assert.Equal(0x80, sa1.ReadRegister(Sfr) & 0x80);
            Assert.False(sa1.ScpuIrqPending);
        }

        [Fact]
        public void Scpu_can_raise_and_the_sa1_can_clear_an_irq()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Cie, 0x80);
            sa1.WriteRegister(Ccnt, 0xA0);

            Assert.Equal(0x80, sa1.ReadRegister(Cfr) & 0x80);

            sa1.WriteRegister(Cic, 0x80);
            Assert.Equal(0x00, sa1.ReadRegister(Cfr) & 0x80);
        }

        // --- Reset handshake ---

        [Fact]
        public void The_sa1_cpu_is_held_in_reset_at_power_on()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Crv, 0x00);
            sa1.WriteRegister((ushort)(Crv + 1), 0x90);

            // RESB is still asserted, so running it must not move the PC.
            ushort before = sa1.Cpu.PC;
            sa1.Run(1000);
            Assert.Equal(before, sa1.Cpu.PC);
        }

        [Fact]
        public void Clearing_resb_starts_the_sa1_cpu_at_crv()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Crv, 0x34);
            sa1.WriteRegister((ushort)(Crv + 1), 0x92);
            sa1.WriteRegister(Ccnt, 0x00);

            Assert.Equal(0x9234, sa1.Cpu.PC);
        }

        [Fact]
        public void Rdyb_parks_the_sa1_cpu_without_resetting_it()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Crv, 0x00);
            sa1.WriteRegister((ushort)(Crv + 1), 0x80);
            sa1.WriteRegister(Ccnt, 0x00);
            sa1.Run(200);
            ushort ran = sa1.Cpu.PC;

            sa1.WriteRegister(Ccnt, 0x40);
            sa1.Run(2000);
            Assert.Equal(ran, sa1.Cpu.PC);
        }

        // The SA-1 CPU fetches its own vectors unconditionally; only the
        // S-CPU's are gated by an enable bit - see Venus_SA1.md §4.3.
        [Fact]
        public void Sa1_side_always_reads_its_own_nmi_and_irq_vectors()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Cnv, 0x11);
            sa1.WriteRegister((ushort)(Cnv + 1), 0x22);
            sa1.WriteRegister(Civ, 0x33);
            sa1.WriteRegister((ushort)(Civ + 1), 0x44);

            Assert.Equal(0x11, sa1.ReadSa1(0x00FFEA));
            Assert.Equal(0x22, sa1.ReadSa1(0x00FFEB));
            Assert.Equal(0x33, sa1.ReadSa1(0x00FFEE));
            Assert.Equal(0x44, sa1.ReadSa1(0x00FFEF));
        }

        [Fact]
        public void Scpu_vectors_are_only_overridden_once_scnt_enables_it()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Snv, 0xAA);
            sa1.WriteRegister(Siv, 0xBB);

            Assert.Equal(CartridgeRegion.Rom, sa1.ResolveScpu(0x00, 0xFFEA).Region);
            Assert.Equal(CartridgeRegion.Rom, sa1.ResolveScpu(0x00, 0xFFEE).Region);

            sa1.WriteRegister(Scnt, 0x30);

            Assert.Equal(CartridgeRegion.Sa1Vector, sa1.ResolveScpu(0x00, 0xFFEA).Region);
            Assert.Equal(CartridgeRegion.Sa1Vector, sa1.ResolveScpu(0x00, 0xFFEE).Region);
            Assert.Equal(0xAA, sa1.VectorByte(sa1.ResolveScpu(0x00, 0xFFEA).Offset));
            Assert.Equal(0xBB, sa1.VectorByte(sa1.ResolveScpu(0x00, 0xFFEE).Offset));
        }

        // The SA-1 side is never redirected by the S-CPU's override bits.
        [Fact]
        public void Scpu_vector_override_does_not_leak_onto_the_sa1_side()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Snv, 0xAA);
            sa1.WriteRegister(Cnv, 0x55);
            sa1.WriteRegister(Scnt, 0x30);

            Assert.Equal(0x55, sa1.ReadSa1(0x00FFEA));
        }
    }
}
