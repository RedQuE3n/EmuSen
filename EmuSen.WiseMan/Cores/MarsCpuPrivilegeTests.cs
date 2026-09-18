using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Which mode the machine is in, and the two things the other two modes may not do - see Mars_Privilege.md.
    public class MarsCpuPrivilegeTests
    {
        private const int Kernel = 0;
        private const int Supervisor = 1;
        private const int User = 2;

        private const ulong Cop0Usable = 1UL << 28;

        [Theory]
        [InlineData(Supervisor)]
        [InlineData(User)]
        public void Coprocessor_zero_is_unusable_outside_kernel_mode(int mode)
        {
            var cpu = PrivilegeFixture.Run(mode, wide: false, 0, a => a.Mfc0(2, Cpu.StatusRegister));

            Assert.Equal(ExceptionCode.CoprocessorUnusable, cpu.LastException!.Code);

            // The corpus checks this: the fault names coprocessor zero by naming no coprocessor at all.
            Assert.Equal(0UL, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseCoprocessor);
        }

        // A Status write with no exception level pending puts the processor in the mode it names at once - see Mars_Performance.md §18.
        [Fact]
        public void Writing_status_enters_the_named_mode_before_the_next_instruction()
        {
            var cpu = new MipsAssembler().Ori(1, 0, (ushort)(User << 3)).Mtc0(1, Cpu.StatusRegister).Mfc0(2, Cpu.StatusRegister).Build();

            cpu.Run(2);
            Assert.Equal(EmuSen.Cores.Nintendo.Mars.Memory.PrivilegeMode.User, cpu.Mode);
            Assert.Null(cpu.LastException);

            // The very next fetch is judged in the new mode, and user mode cannot execute from kseg0.
            cpu.Step();
            Assert.Equal(ExceptionCode.AddressErrorLoad, cpu.LastException!.Code);
            Assert.Equal(MipsAssembler.EntryPoint + 8, cpu.LastException.Address);
        }

        [Fact]
        public void Kernel_mode_needs_no_permission_bit_to_reach_coprocessor_zero()
        {
            var cpu = PrivilegeFixture.Run(Kernel, wide: false, 0, a => a.Mfc0(2, Cpu.StatusRegister));

            Assert.Null(cpu.LastException);
        }

        [Theory]
        [InlineData(Supervisor)]
        [InlineData(User)]
        public void The_permission_bit_grants_what_the_mode_withholds(int mode)
        {
            var cpu = PrivilegeFixture.Run(
                mode, wide: false, 0, a => a.Mfc0(2, Cpu.StatusRegister), status: Cop0Usable);

            Assert.Null(cpu.LastException);
        }

        // An exception runs its handler in kernel mode whatever the mode it interrupted - see §1.
        [Theory]
        [InlineData(Cpu.StatusExceptionLevel)]
        [InlineData(Cpu.StatusErrorLevel)]
        public void Handling_an_exception_is_kernel_mode_whatever_the_mode_field_says(ulong level)
        {
            var cpu = PrivilegeFixture.Run(
                User, wide: false, 0, a => a.Mfc0(2, Cpu.StatusRegister), status: level);

            Assert.Null(cpu.LastException);
        }

        // Every doubleword instruction, from the corpus's own list of twenty-eight - see §4.
        public static TheoryData<string> Wide() => new()
        {
            "dadd", "daddu", "dsub", "dsubu", "daddi", "daddiu",
            "ddiv", "ddivu", "dmult", "dmultu",
            "dsll", "dsrl", "dsra", "dsllv", "dsrlv", "dsrav", "dsll32", "dsrl32", "dsra32",
            "ld", "ldl", "ldr", "sd", "sdl", "sdr", "lld", "scd",
        };

        [Theory]
        [MemberData(nameof(Wide))]
        public void A_doubleword_instruction_is_reserved_in_user_mode_without_its_addressing_bit(string name)
        {
            var cpu = PrivilegeFixture.Run(User, wide: false, 0, a => Emit(a, name));

            Assert.Equal(ExceptionCode.ReservedInstruction, cpu.LastException!.Code);
        }

        [Theory]
        [MemberData(nameof(Wide))]
        public void The_same_instruction_is_reserved_in_supervisor_mode_too(string name)
        {
            var cpu = PrivilegeFixture.Run(Supervisor, wide: false, 0, a => Emit(a, name));

            Assert.Equal(ExceptionCode.ReservedInstruction, cpu.LastException!.Code);
        }

        // The asymmetry worth naming: kernel mode runs them with its own addressing bit clear - see §4.
        [Theory]
        [MemberData(nameof(Wide))]
        public void Kernel_mode_runs_them_with_its_own_addressing_bit_clear(string name)
        {
            var cpu = PrivilegeFixture.Run(Kernel, wide: false, 0, a => Emit(a, name));

            Assert.NotEqual(ExceptionCode.ReservedInstruction, cpu.LastException?.Code);
        }

        [Theory]
        [MemberData(nameof(Wide))]
        public void The_addressing_bit_is_what_the_restriction_reads(string name)
        {
            var cpu = PrivilegeFixture.Run(User, wide: true, 0, a => Emit(a, name));

            Assert.NotEqual(ExceptionCode.ReservedInstruction, cpu.LastException?.Code);
        }

        // The four the corpus lists as not restricted, which produce 64-bit results and are not 64-bit.
        [Theory]
        [InlineData("div")]
        [InlineData("divu")]
        [InlineData("mult")]
        [InlineData("multu")]
        public void An_instruction_that_merely_writes_a_wide_result_is_not_restricted(string name)
        {
            var cpu = PrivilegeFixture.Run(User, wide: false, 0, a => Emit(a, name));

            Assert.Null(cpu.LastException);
        }

        private static MipsAssembler Emit(MipsAssembler a, string name) => name switch
        {
            "dadd" => a.Dadd(2, 3, 4),
            "daddu" => a.Daddu(2, 3, 4),
            "dsub" => a.Dsub(2, 3, 4),
            "dsubu" => a.Dsubu(2, 3, 4),
            "daddi" => a.Daddi(2, 3, 1),
            "daddiu" => a.Daddiu(2, 3, 1),
            "ddiv" => a.Ddiv(3, 4),
            "ddivu" => a.Ddivu(3, 4),
            "dmult" => a.Dmult(3, 4),
            "dmultu" => a.Dmultu(3, 4),
            "dsll" => a.Dsll(2, 3, 1),
            "dsrl" => a.Dsrl(2, 3, 1),
            "dsra" => a.Dsra(2, 3, 1),
            "dsllv" => a.Dsllv(2, 3, 4),
            "dsrlv" => a.Dsrlv(2, 3, 4),
            "dsrav" => a.Dsrav(2, 3, 4),
            "dsll32" => a.Dsll32(2, 3, 1),
            "dsrl32" => a.Dsrl32(2, 3, 1),
            "dsra32" => a.Dsra32(2, 3, 1),
            "ld" => a.Ld(2, 0, 0),
            "ldl" => a.Ldl(2, 0, 0),
            "ldr" => a.Ldr(2, 0, 0),
            "sd" => a.Sd(2, 0, 0),
            "sdl" => a.Sdl(2, 0, 0),
            "sdr" => a.Sdr(2, 0, 0),
            "lld" => a.Lld(2, 0, 0),
            "scd" => a.Scd(2, 0, 0),
            "div" => a.Div(3, 4),
            "divu" => a.Divu(3, 4),
            "mult" => a.Mult(3, 4),
            _ => a.Multu(3, 4),
        };
    }
}
