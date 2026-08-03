using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // `disasm`'s m8/m16/x8/x16 override, through the shell - see `man disasm`.
    public class DisasmWidthHintTests
    {
        // Yoshi's Island $04:FDD8: A0 00 | C5 39 | 10 02 | A0 02 | 84 73 | 85 39.
        private static readonly byte[] Bytes =
        {
            0xA0, 0x00, 0xC5, 0x39, 0x10, 0x02, 0xA0, 0x02, 0x84, 0x73, 0x85, 0x39,
        };

        private static DianaOSInterpreter ShellWithBytesAt(int addr)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            var wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            for (int i = 0; i < Bytes.Length; i++) wram.Write(addr + i, Bytes[i]);
            return DianaShellFixtures.NewShellWithSessions(target).Shell;
        }

        [Fact]
        public void X8_hint_decodes_the_stream_the_cpu_flags_alone_would_get_wrong()
        {
            string outp = ShellWithBytesAt(0x1000).Execute("disasm WRAM 1000 6 m16 x8");

            Assert.Contains("LDY #$00", outp);
            Assert.Contains("STA $39", outp);
            Assert.Contains("decoding with m16 x8", outp);
        }

        // x16 is the direction a reset core's own flags would never give.
        [Fact]
        public void X16_hint_forces_the_misaligned_decode_the_cpu_flags_would_not_give()
        {
            string plain = ShellWithBytesAt(0x1000).Execute("disasm WRAM 1000 6");
            string forced = ShellWithBytesAt(0x1000).Execute("disasm WRAM 1000 6 x16");

            Assert.Contains("STA $39", plain);
            Assert.DoesNotContain("decoding with", plain);

            // Verbatim what the real $04:FDD8 listing looked like.
            Assert.Contains("LDY #$C500", forced);
            Assert.Contains("AND $0210,Y", forced);
            Assert.DoesNotContain("STA $39", forced);
            Assert.Contains("decoding with x16", forced);
        }

        [Fact]
        public void Hints_may_be_written_in_any_order_and_do_not_consume_addr_or_count()
        {
            string outp = ShellWithBytesAt(0x1000).Execute("disasm WRAM x8 1000 m16 6");

            Assert.Contains("001000:", outp);
            Assert.Contains("STA $39", outp);
            Assert.Equal(6, outp.Split('\n').Count(l => l.Contains(": ")));
        }
    }
}
