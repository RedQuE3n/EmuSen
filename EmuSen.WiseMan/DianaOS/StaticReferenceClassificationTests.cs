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
    // IDebugTarget.ClassifyStaticReference (SnesDebugTarget's 65816 opcode
    // table) and the callers/writers/readers shell commands built on top
    // of it - see IDebugTarget.cs's own comment for why this exists: these
    // three commands used to hardcode this exact opcode-byte switch
    // directly in the "core-agnostic" shell layer.
    public class StaticReferenceClassificationTests
    {
        // $008000: JSR $8034 (absolute call, own bank)
        // $008034: STA $0200 (absolute write)
        // $008037: LDA $0200 (absolute read)
        // $00803A: STA $12   (direct page - must NOT resolve, no static target)
        private static SnesDebugTarget BuildTarget()
        {
            byte[] rom = SyntheticRom.Build(
                (0x0000, new byte[] { 0x20, 0x34, 0x80 }),
                (0x0034, new byte[] { 0x8D, 0x00, 0x02 }),
                (0x0037, new byte[] { 0xAD, 0x00, 0x02 }),
                (0x003A, new byte[] { 0x85, 0x12 }));

            var core = SyntheticRom.LoadCore(rom);
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        [Fact]
        public void Jsr_absolute_classifies_as_call_with_correct_target()
        {
            var target = BuildTarget();
            var instr = target.Disassemble("CpuBus", 0x008000, 1).Single();
            var reference = target.ClassifyStaticReference(instr);
            Assert.Equal((StaticReferenceKind.Call, 0x008034), reference);
        }

        [Fact]
        public void Sta_absolute_classifies_as_write_with_correct_target()
        {
            var target = BuildTarget();
            var instr = target.Disassemble("CpuBus", 0x008034, 1).Single();
            var reference = target.ClassifyStaticReference(instr);
            Assert.Equal((StaticReferenceKind.Write, 0x000200), reference);
        }

        [Fact]
        public void Lda_absolute_classifies_as_read_with_correct_target()
        {
            var target = BuildTarget();
            var instr = target.Disassemble("CpuBus", 0x008037, 1).Single();
            var reference = target.ClassifyStaticReference(instr);
            Assert.Equal((StaticReferenceKind.Read, 0x000200), reference);
        }

        [Fact]
        public void Sta_direct_page_has_no_static_target()
        {
            var target = BuildTarget();
            var instr = target.Disassemble("CpuBus", 0x00803A, 1).Single();
            Assert.Null(target.ClassifyStaticReference(instr));
        }

        [Fact]
        public void Callers_command_finds_the_jsr()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            var result = shell.Submit("callers 8034 8000 100");
            Assert.Contains("008000", result.Output);
            Assert.Contains("JSR", result.Output);
        }

        [Fact]
        public void Writers_command_finds_the_sta()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            var result = shell.Submit("writers 200 8000 100");
            Assert.Contains("008034", result.Output);
            Assert.Contains("STA", result.Output);
        }

        [Fact]
        public void Readers_command_finds_the_lda()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            var result = shell.Submit("readers 200 8000 100");
            Assert.Contains("008037", result.Output);
            Assert.Contains("LDA", result.Output);
        }
    }
}
