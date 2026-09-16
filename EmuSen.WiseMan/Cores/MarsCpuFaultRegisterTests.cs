using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // What a fault fills in for the handler, and the link a return breaks - see Mars_Cop0.md §6 and §7.
    public class MarsCpuFaultRegisterTests
    {
        private const ulong Kseg0 = 0xFFFF_FFFF_8000_0000;

        // The address the handler needs is spread over three registers, all derived from the one fault.
        [Fact]
        public void An_address_error_fills_in_the_context_registers_and_not_only_the_address()
        {
            var cpu = Run(3, a => a.Lui(1, 0x8000).Ori(1, 1, 0x2001).Lw(2, 1, 0));

            Assert.Equal(ExceptionCode.AddressErrorLoad, cpu.LastException!.Code);
            Assert.Equal(Kseg0 + 0x2001, cpu.Cop0[Cpu.BadVirtualAddressRegister]);
            Assert.Equal(0x0040_0010UL, cpu.Cop0[Cpu.ContextRegister]);
            Assert.Equal(0x1_FFC0_0010UL, cpu.Cop0[Cpu.XContextRegister]);
        }

        // The upper half of each register belongs to software and a fault must not disturb it.
        [Fact]
        public void A_fault_leaves_the_half_of_those_registers_that_software_owns()
        {
            var cpu = Prepared(c =>
            {
                c.Cop0[Cpu.ContextRegister] = 0xFFFF_FFFF_FFFF_FFFF;
                c.Cop0[Cpu.XContextRegister] = 0xFFFF_FFFF_FFFF_FFFF;
            }, 3, a => a.Lui(1, 0x8000).Ori(1, 1, 0x2001).Lw(2, 1, 0));

            Assert.Equal(0xFFFF_FFFF_FFC0_0010UL, cpu.Cop0[Cpu.ContextRegister]);
            Assert.Equal(0xFFFF_FFFF_FFC0_0010UL, cpu.Cop0[Cpu.XContextRegister]);
        }

        // An address whose upper half is not the sign extension of its own bit 31 is not an address.
        [Fact]
        public void A_load_from_an_address_that_does_not_sign_extend_is_refused()
        {
            var cpu = Run(4, a => a.Lui(1, 0x8000).Dsll32(1, 1, 0).Dsrl32(1, 1, 0).Lw(2, 1, 0));

            Assert.Equal(ExceptionCode.AddressErrorLoad, cpu.LastException!.Code);
        }

        [Fact]
        public void A_store_to_an_address_that_does_not_sign_extend_is_refused()
        {
            var cpu = Run(4, a => a.Lui(1, 0x8000).Dsll32(1, 1, 0).Dsrl32(1, 1, 0).Sw(2, 1, 0));

            Assert.Equal(ExceptionCode.AddressErrorStore, cpu.LastException!.Code);
        }

        // With 64-bit addressing enabled the same address is legal, which is what makes it a mode.
        // The check is a mode, not a rule: with 64-bit addressing on the same address is accepted.
        // Where it then goes is a separate gap - Mars decodes segments from the low word only, so it
        // lands in KSEG0 rather than the 64-bit segment it names. See Mars_Cop0.md §8.1.
        [Fact]
        public void The_width_check_is_a_mode_and_not_a_rule()
        {
            var cpu = Prepared(c => c.Cop0[Cpu.StatusRegister] = Cpu.StatusKernelExtendedAddressing,
                4, a => a.Lui(1, 0x8000).Dsll32(1, 1, 0).Dsrl32(1, 1, 0).Lw(2, 1, 0));

            Assert.Null(cpu.LastException);
        }

        [Fact]
        public void A_cache_operation_checks_its_address_before_doing_nothing()
        {
            var cpu = Run(3, a => a.Lui(1, 0x8000).Ori(1, 1, 0x0002).Cache(0, 1, 0));

            Assert.Equal(ExceptionCode.AddressErrorLoad, cpu.LastException!.Code);
        }

        [Fact]
        public void A_cache_operation_on_an_aligned_address_still_does_nothing()
        {
            var cpu = Run(3, a => a.Lui(1, 0x8000).Ori(1, 1, 0x0004).Cache(0, 1, 0));

            Assert.Null(cpu.LastException);
        }

        // The slot behind an ordinary branch is a delay slot whether or not the branch was taken.
        [Fact]
        public void A_fault_behind_an_untaken_ordinary_branch_saves_the_branch()
        {
            var cpu = Run(4, a => a.Lui(1, 0x8000).Ori(1, 1, 0x2001).Bne(0, 0, 4).Lw(2, 1, 0));

            Assert.Equal(Kseg0 + 8, cpu.Cop0[Cpu.ExceptionPcRegister]);
            Assert.Equal(Cpu.CauseBranchDelay, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseBranchDelay);
        }

        // A likely branch not taken has no delay slot to be in, because the slot never runs.
        [Fact]
        public void The_slot_behind_an_untaken_likely_branch_never_runs_at_all()
        {
            var cpu = Run(4, a => a.Lui(1, 0x8000).Ori(1, 1, 0x2001).Bnel(0, 0, 4).Lw(2, 1, 0).Nop());

            Assert.Null(cpu.LastException);
        }

        [Fact]
        public void A_linked_load_records_the_physical_line_rather_than_the_virtual_address()
        {
            var cpu = Run(3, a => a.Lui(1, 0x8000).Ori(1, 1, 0x0100).Ll(2, 1, 0));

            Assert.Equal(0x10UL, cpu.Cop0[Cpu.LinkedAddressRegister]);
            Assert.True(cpu.LinkedFlag);
        }

        // Returning from a handler breaks the link, but leaves the address a handler can still read.
        [Fact]
        public void A_return_from_exception_breaks_the_link_and_keeps_the_address()
        {
            var cpu = Prepared(c => c.Cop0[Cpu.ExceptionPcRegister] = Kseg0 + 0x40,
                4, a => a.Lui(1, 0x8000).Ori(1, 1, 0x0100).Ll(2, 1, 0).Eret());

            Assert.False(cpu.LinkedFlag);
            Assert.Equal(0x10UL, cpu.Cop0[Cpu.LinkedAddressRegister]);
        }

        [Fact]
        public void A_conditional_store_after_a_return_from_exception_fails()
        {
            var bus = new MarsBus();
            bus.Write32(0x100, 0xDEAD_BEEF);

            var cpu = Prepared(c => c.Cop0[Cpu.ExceptionPcRegister] = Kseg0 + 0x40,
                5, a => a.Lui(1, 0x8000).Ori(1, 1, 0x0100).Ll(2, 1, 0).Eret().Sc(3, 1, 0), bus);

            Assert.Equal(0UL, cpu.Gpr[3]);
            Assert.Equal(0xDEAD_BEEFu, bus.Read32(0x100));
        }

        private static Cpu Run(int steps, Func<MipsAssembler, MipsAssembler> program) =>
            Prepared(_ => { }, steps, program);

        private static Cpu Prepared(
            Action<Cpu> before, int steps, Func<MipsAssembler, MipsAssembler> program, MarsBus? bus = null)
        {
            var cpu = program(new MipsAssembler()).Build(bus);

            before(cpu);
            cpu.Run(steps);

            return cpu;
        }
    }
}
