using EmuSen.Cores;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // `disasm`'s m8/m16/x8/x16 override, through the shell - see `man disasm`.
    public class DisasmWidthHintTests
    {
        // Yoshi's Island $04:FDD8: A0 00 | C5 39 | 10 02 | A0 02 | 84 73 | 85 39.
        private static readonly byte[] Bytes =
        {
            0xA0, 0x00, 0xC5, 0x39, 0x10, 0x02, 0xA0, 0x02, 0x84, 0x73, 0x85, 0x39,
        };

        private static DianaShInterpreter ShellWithBytesAt(int addr)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = SyntheticRom.TargetOf(core);
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
