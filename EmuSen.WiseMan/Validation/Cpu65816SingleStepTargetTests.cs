using EmuSen.Cores.Nintendo.Venus.Validation;

namespace EmuSen.WiseMan.Validation
{
    // The 65816 single-step adapter, whose own ground-truth vectors are
    // third-party data this repo doesn't ship - so nothing else covers it.
    // What matters here is the flat 16MB model: no SNES bank or register
    // decoding may leak in - see Venus_CPU.md §10.1.
    public class Cpu65816SingleStepTargetTests
    {
        // Emulation mode with M/X set, i.e. 8-bit A and index registers.
        private static Cpu65816SingleStepTarget AtPc(ushort pc)
        {
            var t = new Cpu65816SingleStepTarget();
            t.Reset();
            t.SetRegister("e", 1);
            t.SetRegister("p", 0x30);
            t.SetRegister("pbr", 0);
            t.SetRegister("dbr", 0);
            t.SetRegister("s", 0x01FF);
            t.SetRegister("pc", pc);
            return t;
        }

        [Fact]
        public void Every_address_including_register_space_is_plain_ram()
        {
            var t = AtPc(0x8000);
            foreach (int address in new[] { 0x000000, 0x002100, 0x004210, 0x7E0000, 0xFFFFFF })
            {
                t.SetMemory(address, 0x5A);
                Assert.Equal(0x5A, t.GetMemory(address));
            }
        }

        [Fact]
        public void Reads_of_ppu_register_space_return_what_was_stored_there()
        {
            var t = AtPc(0x8000);
            t.SetMemory(0x002100, 0x5A);
            t.SetMemory(0x8000, 0xAD); // LDA $2100
            t.SetMemory(0x8001, 0x00);
            t.SetMemory(0x8002, 0x21);

            t.Step();

            Assert.Equal(0x5A, t.GetRegister("a"));
            Assert.Equal(0x8003, t.GetRegister("pc"));
        }

        [Fact]
        public void Writes_to_ppu_register_space_land_in_memory()
        {
            var t = AtPc(0x8000);
            t.SetRegister("a", 0x7C);
            t.SetMemory(0x8000, 0x8D); // STA $2118
            t.SetMemory(0x8001, 0x18);
            t.SetMemory(0x8002, 0x21);

            t.Step();

            Assert.Equal(0x7C, t.GetMemory(0x002118));
        }

        [Fact]
        public void Reset_clears_memory_and_takes_pc_from_the_flat_reset_vector()
        {
            var t = AtPc(0x8000);
            t.SetMemory(0x001234, 0xFF);
            t.SetMemory(0x00FFFC, 0x34);
            t.SetMemory(0x00FFFD, 0x12);

            t.Reset();

            Assert.Equal(0, t.GetMemory(0x001234));
            Assert.Equal(0, t.GetRegister("pc")); // the vector was cleared too
        }

        [Fact]
        public void Reset_revives_a_cpu_left_halted_by_stp()
        {
            var t = AtPc(0x8000);
            t.SetMemory(0x8000, 0xDB); // STP
            t.Step();

            t.Reset();
            t.SetRegister("e", 1);
            t.SetRegister("p", 0x30);
            t.SetRegister("pc", 0x8000);
            t.SetMemory(0x8000, 0xA9); // LDA #$42
            t.SetMemory(0x8001, 0x42);
            t.Step();

            Assert.Equal(0x42, t.GetRegister("a"));
        }
    }
}
