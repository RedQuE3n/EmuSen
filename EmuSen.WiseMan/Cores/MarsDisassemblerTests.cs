using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Disassembler;
using EmuSen.Cores.Nintendo.Mars.Rsp;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.WiseMan.Cores
{
    // The two pure decoders, word by word against literal text - see Mars_Disassembler.md §8.
    public class MarsDisassemblerTests
    {
        private static string Text(MarsInstruction instruction) =>
            instruction.Operands.Length == 0 ? instruction.Mnemonic : $"{instruction.Mnemonic} {instruction.Operands}";

        private static string Cpu(uint word, uint address = 0x8000_0400) => Text(Vr4300Disassembler.Decode(word, address));

        private static string Rsp(uint word, uint address = 0) => Text(RspDisassembler.Decode(word, address));

        // The integer instructions, one encoding per form; many are words the capstone differential also graded.
        [Theory]
        [InlineData(0x0000_0000u, "nop")]
        [InlineData(0x0005_10C0u, "sll v0, a1, 0x3")]
        [InlineData(0x0005_10C2u, "srl v0, a1, 0x3")]
        [InlineData(0x0005_10C3u, "sra v0, a1, 0x3")]
        [InlineData(0x0085_1004u, "sllv v0, a1, a0")]
        [InlineData(0x0085_1006u, "srlv v0, a1, a0")]
        [InlineData(0x0085_1007u, "srav v0, a1, a0")]
        [InlineData(0x038C_F004u, "sllv s8, t4, gp")]
        [InlineData(0x03E0_0008u, "jr ra")]
        [InlineData(0x0320_F809u, "jalr t9")]
        [InlineData(0x0320_1009u, "jalr v0, t9")]
        [InlineData(0x0060_0009u, "jalr zero, v1")]
        [InlineData(0x0000_000Cu, "syscall")]
        [InlineData(0x0168_A84Cu, "syscall 0x5A2A1")]
        [InlineData(0x0000_000Du, "break")]
        [InlineData(0x000F_894Du, "break 0x3E25")]
        [InlineData(0x0000_000Fu, "sync")]
        [InlineData(0x0000_2010u, "mfhi a0")]
        [InlineData(0x0260_0011u, "mthi s3")]
        [InlineData(0x0000_2012u, "mflo a0")]
        [InlineData(0x0240_0013u, "mtlo s2")]
        [InlineData(0x023A_4814u, "dsllv t1, k0, s1")]
        [InlineData(0x00EC_6816u, "dsrlv t5, t4, a3")]
        [InlineData(0x037F_C017u, "dsrav t8, ra, k1")]
        [InlineData(0x0106_0018u, "mult t0, a2")]
        [InlineData(0x006F_0019u, "multu v1, t7")]
        [InlineData(0x013F_001Au, "div t1, ra")]
        [InlineData(0x030C_001Bu, "divu t8, t4")]
        [InlineData(0x00DC_001Cu, "dmult a2, gp")]
        [InlineData(0x00DC_001Du, "dmultu a2, gp")]
        [InlineData(0x0361_001Eu, "ddiv k1, at")]
        [InlineData(0x0235_001Fu, "ddivu s1, s5")]
        [InlineData(0x00B2_D820u, "add k1, a1, s2")]
        [InlineData(0x00FD_5821u, "addu t3, a3, sp")]
        [InlineData(0x0120_9022u, "sub s2, t1, zero")]
        [InlineData(0x03B2_2823u, "subu a1, sp, s2")]
        [InlineData(0x02E8_0824u, "and at, s7, t0")]
        [InlineData(0x0116_0025u, "or zero, t0, s6")]
        [InlineData(0x028D_6026u, "xor t4, s4, t5")]
        [InlineData(0x0114_E827u, "nor sp, t0, s4")]
        [InlineData(0x01EF_302Au, "slt a2, t7, t7")]
        [InlineData(0x03F8_C02Bu, "sltu t8, ra, t8")]
        [InlineData(0x0092_682Cu, "dadd t5, a0, s2")]
        [InlineData(0x01F1_E02Du, "daddu gp, t7, s1")]
        [InlineData(0x0302_A02Eu, "dsub s4, t8, v0")]
        [InlineData(0x018B_202Fu, "dsubu a0, t4, t3")]
        [InlineData(0x0004_102Fu, "dsubu v0, zero, a0")]
        [InlineData(0x0310_BC70u, "tge t8, s0, 0x2F1")]
        [InlineData(0x03B1_8731u, "tgeu sp, s1, 0x21C")]
        [InlineData(0x019D_2DB2u, "tlt t4, sp, 0xB6")]
        [InlineData(0x01EA_FDF3u, "tltu t7, t2, 0x3F7")]
        [InlineData(0x0085_0034u, "teq a0, a1")]
        [InlineData(0x0298_2CB6u, "tne s4, t8, 0xB2")]
        [InlineData(0x0000_D7B8u, "dsll k0, zero, 0x1E")]
        [InlineData(0x0012_3BBAu, "dsrl a3, s2, 0xE")]
        [InlineData(0x0001_C2BBu, "dsra t8, at, 0xA")]
        [InlineData(0x000D_B77Cu, "dsll32 s6, t5, 0x1D")]
        [InlineData(0x0000_43FEu, "dsrl32 t0, zero, 0xF")]
        [InlineData(0x001A_367Fu, "dsra32 a2, k0, 0x19")]
        [InlineData(0x2035_1E4Fu, "addi s5, at, 0x1E4F")]
        [InlineData(0x27BD_FFE8u, "addiu sp, sp, -0x18")]
        [InlineData(0x2A9F_F2C0u, "slti ra, s4, -0xD40")]
        [InlineData(0x2C85_FFFFu, "sltiu a1, a0, -0x1")]
        [InlineData(0x3085_8010u, "andi a1, a0, 0x8010")]
        [InlineData(0x3405_FFFFu, "ori a1, zero, 0xFFFF")]
        [InlineData(0x3B0E_530Cu, "xori t6, t8, 0x530C")]
        [InlineData(0x3C1A_8000u, "lui k0, 0x8000")]
        [InlineData(0x6333_6086u, "daddi s3, t9, 0x6086")]
        [InlineData(0x67D7_66EDu, "daddiu s7, s8, 0x66ED")]
        [InlineData(0x6405_FFF0u, "daddiu a1, zero, -0x10")]
        public void An_integer_instruction_reads_as_its_listing_does(uint word, string expected)
        {
            Assert.Equal(expected, Cpu(word));
        }

        // Each alias drops an operand that is the zero register; the likely forms have none - see §2.1.
        [Theory]
        [InlineData(0x00A0_2021u, "move a0, a1")]
        [InlineData(0x00A0_2025u, "move a0, a1")]
        [InlineData(0x00A0_202Du, "move a0, a1")]
        [InlineData(0x0000_2025u, "move a0, zero")]
        [InlineData(0x0005_2021u, "addu a0, zero, a1")]
        [InlineData(0x0005_2022u, "neg a0, a1")]
        [InlineData(0x0004_1023u, "negu v0, a0")]
        [InlineData(0x00A0_2027u, "not a0, a1")]
        [InlineData(0x1000_0003u, "b 0x80000410")]
        [InlineData(0x10A0_0004u, "beqz a1, 0x80000414")]
        [InlineData(0x1005_0004u, "beq zero, a1, 0x80000414")]
        [InlineData(0x14A0_0004u, "bnez a1, 0x80000414")]
        [InlineData(0x1400_0004u, "bnez zero, 0x80000414")]
        [InlineData(0x0411_0004u, "bal 0x80000414")]
        [InlineData(0x0413_0004u, "bgezall zero, 0x80000414")]
        [InlineData(0x50A0_0004u, "beql a1, zero, 0x80000414")]
        [InlineData(0x0000_0040u, "sll zero, zero, 0x1")]
        [InlineData(0x0004_0000u, "sll zero, a0, 0x0")]
        public void A_pseudo_instruction_is_used_only_where_the_alias_table_says(uint word, string expected)
        {
            Assert.Equal(expected, Cpu(word));
        }

        [Theory]
        [InlineData(0x07E0_4AFFu, 0xFFFE_6690u, "bltz ra, 0xFFFF9290")]
        [InlineData(0x05C1_6516u, 0x8002_11D0u, "bgez t6, 0x8003A62C")]
        [InlineData(0x05A2_A1B6u, 0x94E0_EE10u, "bltzl t5, 0x94DF74EC")]
        [InlineData(0x05E3_3B99u, 0xC774_F414u, "bgezl t7, 0xC775E27C")]
        [InlineData(0x0730_724Eu, 0x8020_EDE0u, "bltzal t9, 0x8022B71C")]
        [InlineData(0x0511_0004u, 0x8000_0400u, "bgezal t0, 0x80000414")]
        [InlineData(0x06B2_11EFu, 0xFFFC_EFC0u, "bltzall s5, 0xFFFD3780")]
        [InlineData(0x0768_F86Fu, 0x8000_0400u, "tgei k1, -0x791")]
        [InlineData(0x0709_16A4u, 0x8000_0400u, "tgeiu t8, 0x16A4")]
        [InlineData(0x07CA_D960u, 0x8000_0400u, "tlti s8, -0x26A0")]
        [InlineData(0x056B_8613u, 0x8000_0400u, "tltiu t3, -0x79ED")]
        [InlineData(0x062C_8264u, 0x8000_0400u, "teqi s1, -0x7D9C")]
        [InlineData(0x04AE_4D23u, 0x8000_0400u, "tnei a1, 0x4D23")]
        [InlineData(0x1396_82C3u, 0x8B88_F1BCu, "beq gp, s6, 0x8B86FCCC")]
        [InlineData(0x14FD_1A1Fu, 0x9C60_41F8u, "bne a3, sp, 0x9C60AA78")]
        [InlineData(0x52F0_CF9Cu, 0x8044_8E74u, "beql s7, s0, 0x8043CCE8")]
        [InlineData(0x5490_7D19u, 0xFFFD_0C94u, "bnel a0, s0, 0xFFFF00FC")]
        [InlineData(0x1B80_2427u, 0x0001_5960u, "blez gp, 0x0001EA00")]
        [InlineData(0x1C20_C696u, 0x7FFF_FFF8u, "bgtz at, 0x7FFF1A54")]
        [InlineData(0x5A00_B754u, 0xFFFF_FFFCu, "blezl s0, 0xFFFEDD50")]
        [InlineData(0x5FA0_3C3Fu, 0x6FFF_FFFCu, "bgtzl sp, 0x7000F0FC")]
        public void A_branch_and_a_register_immediate_read_as_their_listing_does(uint word, uint address, string expected)
        {
            Assert.Equal(expected, Cpu(word, address));
        }

        // The offset counts from the delay slot, is sign-extended, and the sum wraps at thirty-two bits - see §4.
        [Theory]
        [InlineData(0x1000_FFFFu, 0x8000_0400u, "b 0x80000400")]
        [InlineData(0x1485_8000u, 0x8000_0400u, "bne a0, a1, 0x7FFE0404")]
        [InlineData(0x1085_8000u, 0x0000_0010u, "beq a0, a1, 0xFFFE0014")]
        [InlineData(0x53F5_2D4Fu, 0xFFFF_FFFCu, "beql ra, s5, 0x0000B53C")]
        [InlineData(0x1085_7FFFu, 0x8000_0000u, "beq a0, a1, 0x80020000")]
        public void A_branch_target_is_the_delay_slot_plus_the_signed_offset_times_four(uint word, uint address, string expected)
        {
            Assert.Equal(expected, Cpu(word, address));
        }

        // The region is the delay slot's, so a jump in a region's last word lands in the next one - see §4.
        [Theory]
        [InlineData(0x0B65_4E12u, 0x8026_8D6Cu, "j 0x8D953848")]
        [InlineData(0x0C00_0100u, 0x8000_0400u, "jal 0x80000400")]
        [InlineData(0x0800_0000u, 0x8FFF_FFFCu, "j 0x90000000")]
        [InlineData(0x0800_0000u, 0x8FFF_FFF8u, "j 0x80000000")]
        [InlineData(0x0FFF_FFFFu, 0xA000_0000u, "jal 0xAFFFFFFC")]
        [InlineData(0x0E92_E519u, 0x0001_FEC0u, "jal 0x0A4B9464")]
        [InlineData(0x0800_0001u, 0xFFFF_FFFCu, "j 0x00000004")]
        public void A_jump_keeps_the_top_four_bits_of_its_delay_slot(uint word, uint address, string expected)
        {
            Assert.Equal(expected, Cpu(word, address));
        }

        [Theory]
        [InlineData(0xDFBF_0010u, "ld ra, 0x10(sp)")]
        [InlineData(0xFFBF_0010u, "sd ra, 0x10(sp)")]
        [InlineData(0x8CCA_A905u, "lw t2, -0x56FB(a2)")]
        [InlineData(0x8C85_0000u, "lw a1, 0x0(a0)")]
        [InlineData(0x80A4_0004u, "lb a0, 0x4(a1)")]
        [InlineData(0x91C0_10DDu, "lbu zero, 0x10DD(t6)")]
        [InlineData(0x8573_F526u, "lh s3, -0xADA(t3)")]
        [InlineData(0x9472_F419u, "lhu s2, -0xBE7(v1)")]
        [InlineData(0x9CE3_4A0Du, "lwu v1, 0x4A0D(a3)")]
        [InlineData(0x8ABF_734Eu, "lwl ra, 0x734E(s5)")]
        [InlineData(0x9B9A_C3D3u, "lwr k0, -0x3C2D(gp)")]
        [InlineData(0x6ABC_E5E0u, "ldl gp, -0x1A20(s5)")]
        [InlineData(0x6FC0_270Au, "ldr zero, 0x270A(s8)")]
        [InlineData(0xA8D0_1832u, "swl s0, 0x1832(a2)")]
        [InlineData(0xBB90_2B4Bu, "swr s0, 0x2B4B(gp)")]
        [InlineData(0xB271_5C65u, "sdl s1, 0x5C65(s3)")]
        [InlineData(0xB413_46F2u, "sdr s3, 0x46F2(zero)")]
        [InlineData(0xC0D1_29C9u, "ll s1, 0x29C9(a2)")]
        [InlineData(0xE368_7BAEu, "sc t0, 0x7BAE(k1)")]
        [InlineData(0xD101_9BF8u, "lld at, -0x6408(t0)")]
        [InlineData(0xF0C2_155Fu, "scd v0, 0x155F(a2)")]
        [InlineData(0xA1BA_FEF1u, "sb k0, -0x10F(t5)")]
        [InlineData(0xA452_96B9u, "sh s2, -0x6947(v0)")]
        [InlineData(0xAF50_D8FDu, "sw s0, -0x2703(k0)")]
        [InlineData(0xBC19_0010u, "cache 0x19, 0x10(zero)")]
        [InlineData(0xBFD1_B208u, "cache 0x11, -0x4DF8(s8)")]
        [InlineData(0xC459_B974u, "lwc1 f25, -0x468C(v0)")]
        [InlineData(0xD56B_2DBFu, "ldc1 f11, 0x2DBF(t3)")]
        [InlineData(0xE6A2_48B3u, "swc1 f2, 0x48B3(s5)")]
        [InlineData(0xF628_5F05u, "sdc1 f8, 0x5F05(s1)")]
        public void A_load_or_store_reads_as_register_offset_and_base(uint word, string expected)
        {
            Assert.Equal(expected, Cpu(word));
        }

        [Theory]
        [InlineData(0x4080_6000u, "mtc0 zero, Status")]
        [InlineData(0x4018_5800u, "mfc0 t8, Compare")]
        [InlineData(0x4024_D800u, "dmfc0 a0, CacheErr")]
        [InlineData(0x40BB_4800u, "dmtc0 k1, Count")]
        [InlineData(0x405F_2000u, "cfc0 ra, $4")]
        [InlineData(0x40C2_F800u, "ctc0 v0, $31")]
        [InlineData(0x4200_0001u, "tlbr")]
        [InlineData(0x4200_0002u, "tlbwi")]
        [InlineData(0x4200_0006u, "tlbwr")]
        [InlineData(0x4200_0008u, "tlbp")]
        [InlineData(0x4200_0018u, "eret")]
        [InlineData(0x43FF_FFD8u, "eret")]
        [InlineData(0x42FF_ADC1u, "tlbr")]
        public void A_coprocessor_zero_instruction_names_its_register(uint word, string expected)
        {
            Assert.Equal(expected, Cpu(word));
        }

        // Every index, so a shifted or swapped name cannot pass - see Mars_Cop0.md for what each holds.
        [Theory]
        [InlineData(0, "Index")]
        [InlineData(1, "Random")]
        [InlineData(2, "EntryLo0")]
        [InlineData(3, "EntryLo1")]
        [InlineData(4, "Context")]
        [InlineData(5, "PageMask")]
        [InlineData(6, "Wired")]
        [InlineData(7, "$7")]
        [InlineData(8, "BadVAddr")]
        [InlineData(9, "Count")]
        [InlineData(10, "EntryHi")]
        [InlineData(11, "Compare")]
        [InlineData(12, "Status")]
        [InlineData(13, "Cause")]
        [InlineData(14, "EPC")]
        [InlineData(15, "PRId")]
        [InlineData(16, "Config")]
        [InlineData(17, "LLAddr")]
        [InlineData(18, "WatchLo")]
        [InlineData(19, "WatchHi")]
        [InlineData(20, "XContext")]
        [InlineData(21, "$21")]
        [InlineData(22, "$22")]
        [InlineData(23, "$23")]
        [InlineData(24, "$24")]
        [InlineData(25, "$25")]
        [InlineData(26, "PErr")]
        [InlineData(27, "CacheErr")]
        [InlineData(28, "TagLo")]
        [InlineData(29, "TagHi")]
        [InlineData(30, "ErrorEPC")]
        [InlineData(31, "$31")]
        public void Every_coprocessor_zero_index_has_its_manual_name(int index, string expected)
        {
            Assert.Equal($"mfc0 at, {expected}", Cpu(0x4001_0000u | ((uint)index << 11)));
        }

        [Theory]
        [InlineData(0x4413_A000u, "mfc1 s3, f20")]
        [InlineData(0x443B_B000u, "dmfc1 k1, f22")]
        [InlineData(0x445F_7800u, "cfc1 ra, fcr15")]
        [InlineData(0x4485_9000u, "mtc1 a1, f18")]
        [InlineData(0x44A1_0800u, "dmtc1 at, f1")]
        [InlineData(0x44C1_F800u, "ctc1 at, fcr31")]
        [InlineData(0x4500_0004u, "bc1f 0x80000414")]
        [InlineData(0x4501_0004u, "bc1t 0x80000414")]
        [InlineData(0x4502_0004u, "bc1fl 0x80000414")]
        [InlineData(0x4503_0004u, "bc1tl 0x80000414")]
        [InlineData(0x4100_0004u, "bc0f 0x80000414")]
        [InlineData(0x4103_0004u, "bc0tl 0x80000414")]
        [InlineData(0x4602_08C0u, "add.s f3, f1, f2")]
        [InlineData(0x4612_A281u, "sub.s f10, f20, f18")]
        [InlineData(0x4638_0C02u, "mul.d f16, f1, f24")]
        [InlineData(0x4607_D383u, "div.s f14, f26, f7")]
        [InlineData(0x4620_D384u, "sqrt.d f14, f26")]
        [InlineData(0x4620_1105u, "abs.d f4, f2")]
        [InlineData(0x4600_F506u, "mov.s f20, f30")]
        [InlineData(0x4600_8307u, "neg.s f12, f16")]
        [InlineData(0x4600_EF08u, "round.l.s f28, f29")]
        [InlineData(0x4620_72C9u, "trunc.l.d f11, f14")]
        [InlineData(0x4600_9ECAu, "ceil.l.s f27, f19")]
        [InlineData(0x4620_448Bu, "floor.l.d f18, f8")]
        [InlineData(0x4600_D54Cu, "round.w.s f21, f26")]
        [InlineData(0x4620_5BCDu, "trunc.w.d f15, f11")]
        [InlineData(0x4600_68CEu, "ceil.w.s f3, f13")]
        [InlineData(0x4620_99CFu, "floor.w.d f7, f19")]
        [InlineData(0x4620_D760u, "cvt.s.d f29, f26")]
        [InlineData(0x4600_D021u, "cvt.d.s f0, f26")]
        [InlineData(0x4600_FEE4u, "cvt.w.s f27, f31")]
        [InlineData(0x4620_C365u, "cvt.l.d f13, f24")]
        [InlineData(0x4680_62A0u, "cvt.s.w f10, f12")]
        [InlineData(0x4680_6421u, "cvt.d.w f16, f12")]
        [InlineData(0x46A0_3920u, "cvt.s.l f4, f7")]
        [InlineData(0x46A0_7221u, "cvt.d.l f8, f14")]
        [InlineData(0x4602_083Cu, "c.lt.s f1, f2")]
        [InlineData(0x463D_2030u, "c.f.d f4, f29")]
        [InlineData(0x4631_B03Fu, "c.ngt.d f22, f17")]
        [InlineData(0x4819_B000u, "mfc2 t9, $22")]
        [InlineData(0x483F_2800u, "dmfc2 ra, $5")]
        [InlineData(0x4854_4800u, "cfc2 s4, $9")]
        [InlineData(0x488A_1800u, "mtc2 t2, $3")]
        [InlineData(0x48AC_7800u, "dmtc2 t4, $15")]
        [InlineData(0x48C5_6000u, "ctc2 a1, $12")]
        public void A_coprocessor_one_or_two_instruction_reads_as_its_listing_does(uint word, string expected)
        {
            Assert.Equal(expected, Cpu(word));
        }

        // The sixteen conditions in order, so a slipped table index fails one of them - see Mars_FpuMath.md §7.
        [Theory]
        [InlineData(0x0, "f")]
        [InlineData(0x1, "un")]
        [InlineData(0x2, "eq")]
        [InlineData(0x3, "ueq")]
        [InlineData(0x4, "olt")]
        [InlineData(0x5, "ult")]
        [InlineData(0x6, "ole")]
        [InlineData(0x7, "ule")]
        [InlineData(0x8, "sf")]
        [InlineData(0x9, "ngle")]
        [InlineData(0xA, "seq")]
        [InlineData(0xB, "ngl")]
        [InlineData(0xC, "lt")]
        [InlineData(0xD, "nge")]
        [InlineData(0xE, "le")]
        [InlineData(0xF, "ngt")]
        public void Every_compare_condition_has_its_name(uint condition, string expected)
        {
            Assert.Equal($"c.{expected}.s f1, f2", Cpu(0x4602_0830u | condition));
        }

        // What the interpreter refuses or does not own is a data word, whatever a later ISA calls it - see §3.
        [Theory]
        [InlineData(0x0000_0001u)]
        [InlineData(0x0000_0005u)]
        [InlineData(0x0085_100Au)]
        [InlineData(0x0085_100Bu)]
        [InlineData(0x0000_000Eu)]
        [InlineData(0x0000_0015u)]
        [InlineData(0x0000_0028u)]
        [InlineData(0x0000_0035u)]
        [InlineData(0x0000_0037u)]
        [InlineData(0x0000_003Du)]
        [InlineData(0x0484_0010u)]
        [InlineData(0x048D_0010u)]
        [InlineData(0x4065_6000u)]
        [InlineData(0x4200_0010u)]
        [InlineData(0x4200_001Fu)]
        [InlineData(0x4104_0004u)]
        [InlineData(0x4465_6000u)]
        [InlineData(0x4504_0004u)]
        [InlineData(0x4600_08A0u)]
        [InlineData(0x4620_08A1u)]
        [InlineData(0x4680_0000u)]
        [InlineData(0x46A0_0024u)]
        [InlineData(0x4600_0010u)]
        [InlineData(0x4620_0022u)]
        [InlineData(0x4640_0000u)]
        [InlineData(0x4905_6000u)]
        [InlineData(0x4C00_0000u)]
        [InlineData(0x7000_0000u)]
        [InlineData(0x7430_9BDCu)]
        [InlineData(0xC885_0010u)]
        [InlineData(0xCC85_8010u)]
        [InlineData(0xD885_0010u)]
        [InlineData(0xE885_0010u)]
        [InlineData(0xEC85_8010u)]
        [InlineData(0xF885_0010u)]
        public void An_encoding_the_interpreter_does_not_execute_is_a_word(uint word)
        {
            var instruction = Vr4300Disassembler.Decode(word, 0x8000_0400);

            Assert.Equal(".word", instruction.Mnemonic);
            Assert.Equal($"0x{word:X8}", instruction.Operands);
            Assert.Null(instruction.Kind);
        }

        [Fact]
        public void A_data_word_prints_all_eight_digits()
        {
            Assert.Equal(".word 0x04840010", Cpu(0x0484_0010));
        }

        // Every control transfer with a target in the word is a call reference, as the other cores' are - see §4.
        [Theory]
        [InlineData(0x0C00_0100u, 0x8000_0400u, 0x8000_0400u)]
        [InlineData(0x0800_0000u, 0x8FFF_FFFCu, 0x9000_0000u)]
        [InlineData(0x1085_8000u, 0x0000_0010u, 0xFFFE_0014u)]
        [InlineData(0x0411_0004u, 0x8000_0400u, 0x8000_0414u)]
        [InlineData(0x5490_7D19u, 0xFFFD_0C94u, 0xFFFF_00FCu)]
        [InlineData(0x4503_0004u, 0x8000_0400u, 0x8000_0414u)]
        public void A_branch_or_jump_names_its_target_as_a_call(uint word, uint address, uint target)
        {
            var instruction = Vr4300Disassembler.Decode(word, address);

            Assert.Equal(StaticReferenceKind.Call, instruction.Kind);
            Assert.Equal(target, instruction.Target);
        }

        // Only a base of zero makes the address a constant, and the offset is sign-extended to reach it - see §4.
        [Theory]
        [InlineData(0x8C04_FFF0u, StaticReferenceKind.Read, 0xFFFF_FFF0u)]
        [InlineData(0xAC04_0100u, StaticReferenceKind.Write, 0x0000_0100u)]
        [InlineData(0xB413_46F2u, StaticReferenceKind.Write, 0x0000_46F2u)]
        [InlineData(0xDC04_8000u, StaticReferenceKind.Read, 0xFFFF_8000u)]
        [InlineData(0xC004_0008u, StaticReferenceKind.Read, 0x0000_0008u)]
        [InlineData(0xE004_0008u, StaticReferenceKind.Write, 0x0000_0008u)]
        [InlineData(0xC404_0010u, StaticReferenceKind.Read, 0x0000_0010u)]
        [InlineData(0xF404_0010u, StaticReferenceKind.Write, 0x0000_0010u)]
        [InlineData(0x6804_7FFFu, StaticReferenceKind.Read, 0x0000_7FFFu)]
        public void An_access_based_on_zero_names_its_address(uint word, StaticReferenceKind kind, uint target)
        {
            var instruction = Vr4300Disassembler.Decode(word, 0x8000_0400);

            Assert.Equal(kind, instruction.Kind);
            Assert.Equal(target, instruction.Target);
        }

        [Theory]
        [InlineData(0x8FA4_0010u)]
        [InlineData(0xAFA4_0010u)]
        [InlineData(0x03E0_0008u)]
        [InlineData(0x0320_F809u)]
        [InlineData(0xBC19_0010u)]
        [InlineData(0x2404_0010u)]
        [InlineData(0x3C04_8000u)]
        [InlineData(0x4080_6000u)]
        [InlineData(0x4200_0018u)]
        [InlineData(0x0000_000Cu)]
        public void An_instruction_with_no_constant_address_names_nothing(uint word)
        {
            var instruction = Vr4300Disassembler.Decode(word, 0x8000_0400);

            Assert.Null(instruction.Kind);
            Assert.Equal(0u, instruction.Target);
        }

        // The RSP's scalar half, which is MIPS I without the parts it lacks - see Mars_Rsp.md §1.
        [Theory]
        [InlineData(0x0000_0000u, "nop")]
        [InlineData(0x0005_10C0u, "sll v0, a1, 0x3")]
        [InlineData(0x0005_10C2u, "srl v0, a1, 0x3")]
        [InlineData(0x0005_10C3u, "sra v0, a1, 0x3")]
        [InlineData(0x0085_1004u, "sllv v0, a1, a0")]
        [InlineData(0x0085_1006u, "srlv v0, a1, a0")]
        [InlineData(0x0085_1007u, "srav v0, a1, a0")]
        [InlineData(0x03E0_0008u, "jr ra")]
        [InlineData(0x0020_F809u, "jalr at")]
        [InlineData(0x0020_0809u, "jalr at, at")]
        [InlineData(0x0000_000Du, "break")]
        [InlineData(0x0085_1020u, "add v0, a0, a1")]
        [InlineData(0x0085_1021u, "addu v0, a0, a1")]
        [InlineData(0x0085_1022u, "sub v0, a0, a1")]
        [InlineData(0x0085_1023u, "subu v0, a0, a1")]
        [InlineData(0x0085_1024u, "and v0, a0, a1")]
        [InlineData(0x0085_1025u, "or v0, a0, a1")]
        [InlineData(0x0085_1026u, "xor v0, a0, a1")]
        [InlineData(0x0085_1027u, "nor v0, a0, a1")]
        [InlineData(0x0085_102Au, "slt v0, a0, a1")]
        [InlineData(0x0085_102Bu, "sltu v0, a0, a1")]
        [InlineData(0x0080_1021u, "move v0, a0")]
        [InlineData(0x2004_FFFFu, "addi a0, zero, -0x1")]
        [InlineData(0x2484_0010u, "addiu a0, a0, 0x10")]
        [InlineData(0x2885_8000u, "slti a1, a0, -0x8000")]
        [InlineData(0x2C85_0001u, "sltiu a1, a0, 0x1")]
        [InlineData(0x3085_FFFFu, "andi a1, a0, 0xFFFF")]
        [InlineData(0x3485_0F00u, "ori a1, a0, 0xF00")]
        [InlineData(0x3885_0001u, "xori a1, a0, 0x1")]
        [InlineData(0x3C01_1234u, "lui at, 0x1234")]
        [InlineData(0x8085_0003u, "lb a1, 0x3(a0)")]
        [InlineData(0x8485_FFFEu, "lh a1, -0x2(a0)")]
        [InlineData(0x8C85_0FFCu, "lw a1, 0xFFC(a0)")]
        [InlineData(0x9085_0001u, "lbu a1, 0x1(a0)")]
        [InlineData(0x9485_0002u, "lhu a1, 0x2(a0)")]
        [InlineData(0x9C04_0010u, "lwu a0, 0x10(zero)")]
        [InlineData(0xA085_0003u, "sb a1, 0x3(a0)")]
        [InlineData(0xA485_0002u, "sh a1, 0x2(a0)")]
        [InlineData(0xAC85_0004u, "sw a1, 0x4(a0)")]
        public void An_rsp_scalar_instruction_reads_as_its_listing_does(uint word, string expected)
        {
            Assert.Equal(expected, Rsp(word));
        }

        // The RSP repeats the alias rules and the code field rather than sharing them, so they are pinned here too - see §8.
        [Theory]
        [InlineData(0x0005_2022u, "neg a0, a1")]
        [InlineData(0x0004_1023u, "negu v0, a0")]
        [InlineData(0x00A0_2027u, "not a0, a1")]
        [InlineData(0x0005_2027u, "nor a0, zero, a1")]
        [InlineData(0x00A0_2025u, "move a0, a1")]
        [InlineData(0x0005_2025u, "or a0, zero, a1")]
        [InlineData(0x0005_2021u, "addu a0, zero, a1")]
        [InlineData(0x000F_894Du, "break 0x3E25")]
        public void An_rsp_alias_or_code_follows_the_same_rules_as_the_vr4300s(uint word, string expected)
        {
            Assert.Equal(expected, Rsp(word));
        }

        // IMEM is twelve bits, and both a jump and a branch stay inside it - see Mars_Rsp.md §2.
        [Theory]
        [InlineData(0x0800_0400u, 0x000u, "j 0x000")]
        [InlineData(0x0800_03FFu, 0x000u, "j 0xFFC")]
        [InlineData(0x0800_0401u, 0x000u, "j 0x004")]
        [InlineData(0x0C00_0040u, 0x800u, "jal 0x100")]
        [InlineData(0x1085_0003u, 0xFF8u, "beq a0, a1, 0x008")]
        [InlineData(0x1485_FFFEu, 0x004u, "bne a0, a1, 0x000")]
        [InlineData(0x1485_FFF0u, 0x010u, "bne a0, a1, 0xFD4")]
        [InlineData(0x1000_0001u, 0x100u, "b 0x108")]
        [InlineData(0x1080_0001u, 0x100u, "beqz a0, 0x108")]
        [InlineData(0x1480_0001u, 0x100u, "bnez a0, 0x108")]
        [InlineData(0x1880_0001u, 0x100u, "blez a0, 0x108")]
        [InlineData(0x1C80_0001u, 0x100u, "bgtz a0, 0x108")]
        [InlineData(0x0480_0004u, 0x000u, "bltz a0, 0x014")]
        [InlineData(0x0481_0004u, 0x000u, "bgez a0, 0x014")]
        [InlineData(0x0490_0004u, 0x000u, "bltzal a0, 0x014")]
        [InlineData(0x0491_0004u, 0x000u, "bgezal a0, 0x014")]
        [InlineData(0x0411_0004u, 0x000u, "bal 0x014")]
        [InlineData(0x1085_0003u, 0x0400_1FF8u, "beq a0, a1, 0x008")]
        public void An_rsp_branch_or_jump_lands_inside_instruction_memory(uint word, uint address, string expected)
        {
            var instruction = RspDisassembler.Decode(word, address);

            Assert.Equal(expected, Text(instruction));
            Assert.Equal(StaticReferenceKind.Call, instruction.Kind);
            Assert.Equal(Convert.ToUInt32(expected[^3..], 16), instruction.Target);
        }

        [Theory]
        [InlineData(0x4008_2000u, "mfc0 t0, SP_STATUS")]
        [InlineData(0x4088_4800u, "mtc0 t0, DPC_END")]
        [InlineData(0x4008_A000u, "mfc0 t0, SP_STATUS")]
        [InlineData(0x4808_1A00u, "mfc2 t0, v3[4]")]
        [InlineData(0x4888_FF80u, "mtc2 t0, v31[15]")]
        [InlineData(0x4848_0000u, "cfc2 t0, vco")]
        [InlineData(0x4848_0800u, "cfc2 t0, vcc")]
        [InlineData(0x4848_1000u, "cfc2 t0, vce")]
        [InlineData(0x4848_1800u, "cfc2 t0, vce")]
        [InlineData(0x4848_2800u, "cfc2 t0, vcc")]
        [InlineData(0x48C8_1000u, "ctc2 t0, vce")]
        public void An_rsp_transfer_names_its_register_the_way_the_interpreter_reads_it(uint word, string expected)
        {
            Assert.Equal(expected, Rsp(word));
        }

        [Theory]
        [InlineData(0, "SP_MEM_ADDR")]
        [InlineData(1, "SP_DRAM_ADDR")]
        [InlineData(2, "SP_RD_LEN")]
        [InlineData(3, "SP_WR_LEN")]
        [InlineData(4, "SP_STATUS")]
        [InlineData(5, "SP_DMA_FULL")]
        [InlineData(6, "SP_DMA_BUSY")]
        [InlineData(7, "SP_SEMAPHORE")]
        [InlineData(8, "DPC_START")]
        [InlineData(9, "DPC_END")]
        [InlineData(10, "DPC_CURRENT")]
        [InlineData(11, "DPC_STATUS")]
        [InlineData(12, "DPC_CLOCK")]
        [InlineData(13, "DPC_BUFBUSY")]
        [InlineData(14, "DPC_PIPEBUSY")]
        [InlineData(15, "DPC_TMEM")]
        public void Every_rsp_coprocessor_zero_index_has_its_interface_name(int index, string expected)
        {
            Assert.Equal($"mfc0 t0, {expected}", Rsp(0x4008_0000u | ((uint)index << 11)));
        }

        // The selector notation, one encoding per spelling - see §5.1.
        [Theory]
        [InlineData(0x4A03_1040u, "vmulf v1, v2, v3")]
        [InlineData(0x4A23_1040u, "vmulf v1, v2, v3")]
        [InlineData(0x4A43_1048u, "vmacf v1, v2, v3[0q]")]
        [InlineData(0x4A63_1050u, "vadd v1, v2, v3[1q]")]
        [InlineData(0x4A83_1051u, "vsub v1, v2, v3[0h]")]
        [InlineData(0x4AA3_1053u, "vabs v1, v2, v3[1h]")]
        [InlineData(0x4AC3_1068u, "vand v1, v2, v3[2h]")]
        [InlineData(0x4AE3_1067u, "vmrg v1, v2, v3[3h]")]
        [InlineData(0x4B03_1047u, "vmudh v1, v2, v3[0]")]
        [InlineData(0x4B23_105Du, "vsar v1, v2, v3[1]")]
        [InlineData(0x4BE3_106Cu, "vxor v1, v2, v3[7]")]
        [InlineData(0x4B62_2870u, "vrcp v1[5], v2[3]")]
        [InlineData(0x4B04_0133u, "vmov v4[0], v4[0]")]
        [InlineData(0x4B86_68B6u, "vrsqh v2[5], v6[4]")]
        [InlineData(0x4A03_1071u, "vrcpl v1[2], v3")]
        [InlineData(0x4A00_0037u, "vnop")]
        [InlineData(0x4BFF_FFFFu, "vnull")]
        public void An_rsp_vector_operation_spells_its_selector_the_sgi_way(uint word, string expected)
        {
            Assert.Equal(expected, Rsp(word));
        }

        // Every function, so a shifted name table cannot pass; null is the five no reference names - see §7.2.
        [Theory]
        [InlineData(0x00, "vmulf")]
        [InlineData(0x01, "vmulu")]
        [InlineData(0x02, "vrndp")]
        [InlineData(0x03, "vmulq")]
        [InlineData(0x04, "vmudl")]
        [InlineData(0x05, "vmudm")]
        [InlineData(0x06, "vmudn")]
        [InlineData(0x07, "vmudh")]
        [InlineData(0x08, "vmacf")]
        [InlineData(0x09, "vmacu")]
        [InlineData(0x0A, "vrndn")]
        [InlineData(0x0B, "vmacq")]
        [InlineData(0x0C, "vmadl")]
        [InlineData(0x0D, "vmadm")]
        [InlineData(0x0E, "vmadn")]
        [InlineData(0x0F, "vmadh")]
        [InlineData(0x10, "vadd")]
        [InlineData(0x11, "vsub")]
        [InlineData(0x12, "vsut")]
        [InlineData(0x13, "vabs")]
        [InlineData(0x14, "vaddc")]
        [InlineData(0x15, "vsubc")]
        [InlineData(0x16, "vaddb")]
        [InlineData(0x17, "vsubb")]
        [InlineData(0x18, "vaccb")]
        [InlineData(0x19, "vsucb")]
        [InlineData(0x1A, "vsad")]
        [InlineData(0x1B, "vsac")]
        [InlineData(0x1C, "vsum")]
        [InlineData(0x1D, "vsar")]
        [InlineData(0x1E, null)]
        [InlineData(0x1F, null)]
        [InlineData(0x20, "vlt")]
        [InlineData(0x21, "veq")]
        [InlineData(0x22, "vne")]
        [InlineData(0x23, "vge")]
        [InlineData(0x24, "vcl")]
        [InlineData(0x25, "vch")]
        [InlineData(0x26, "vcr")]
        [InlineData(0x27, "vmrg")]
        [InlineData(0x28, "vand")]
        [InlineData(0x29, "vnand")]
        [InlineData(0x2A, "vor")]
        [InlineData(0x2B, "vnor")]
        [InlineData(0x2C, "vxor")]
        [InlineData(0x2D, "vnxor")]
        [InlineData(0x2E, null)]
        [InlineData(0x2F, null)]
        [InlineData(0x30, "vrcp")]
        [InlineData(0x31, "vrcpl")]
        [InlineData(0x32, "vrcph")]
        [InlineData(0x33, "vmov")]
        [InlineData(0x34, "vrsq")]
        [InlineData(0x35, "vrsql")]
        [InlineData(0x36, "vrsqh")]
        [InlineData(0x37, "vnop")]
        [InlineData(0x38, "vextt")]
        [InlineData(0x39, "vextq")]
        [InlineData(0x3A, "vextn")]
        [InlineData(0x3B, null)]
        [InlineData(0x3C, "vinst")]
        [InlineData(0x3D, "vinsq")]
        [InlineData(0x3E, "vinsn")]
        [InlineData(0x3F, "vnull")]
        public void Every_rsp_vector_function_has_its_reference_name(uint funct, string? expected)
        {
            uint word = 0x4A03_1040u | funct;
            var instruction = RspDisassembler.Decode(word, 0);

            Assert.Equal(expected ?? ".word", instruction.Mnemonic);
            Assert.Null(instruction.Kind);
        }

        // The seven-bit offset is sign-extended, then scaled by the format's own size - see §5.3.
        [Theory]
        [InlineData(0xC881_2001u, "lqv v1[0], 0x10(a0)")]
        [InlineData(0xEAA3_207Fu, "sqv v3[0], -0x10(s5)")]
        [InlineData(0xC802_07C0u, "lbv v2[15], -0x40(zero)")]
        [InlineData(0xC905_0903u, "lsv v5[2], 0x6(t0)")]
        [InlineData(0xC806_127Fu, "llv v6[4], -0x4(zero)")]
        [InlineData(0xC801_1803u, "ldv v1[0], 0x18(zero)")]
        [InlineData(0xC8A7_2802u, "lrv v7[0], 0x20(a1)")]
        [InlineData(0xC808_3001u, "lpv v8[0], 0x8(zero)")]
        [InlineData(0xC889_387Eu, "luv v9[0], -0x10(a0)")]
        [InlineData(0xC88A_4001u, "lhv v10[0], 0x10(a0)")]
        [InlineData(0xC88B_4C01u, "lfv v11[8], 0x10(a0)")]
        [InlineData(0xC890_5901u, "ltv v16[2], 0x10(a0)")]
        [InlineData(0xE801_0001u, "sbv v1[0], 0x1(zero)")]
        [InlineData(0xE801_0801u, "ssv v1[0], 0x2(zero)")]
        [InlineData(0xE801_1001u, "slv v1[0], 0x4(zero)")]
        [InlineData(0xE801_1801u, "sdv v1[0], 0x8(zero)")]
        [InlineData(0xE801_2001u, "sqv v1[0], 0x10(zero)")]
        [InlineData(0xE801_2801u, "srv v1[0], 0x10(zero)")]
        [InlineData(0xE801_3001u, "spv v1[0], 0x8(zero)")]
        [InlineData(0xE801_3801u, "suv v1[0], 0x8(zero)")]
        [InlineData(0xE801_4001u, "shv v1[0], 0x10(zero)")]
        [InlineData(0xE801_4801u, "sfv v1[0], 0x10(zero)")]
        [InlineData(0xE801_5001u, "swv v1[0], 0x10(zero)")]
        [InlineData(0xE801_5801u, "stv v1[0], 0x10(zero)")]
        [InlineData(0xE801_203Fu, "sqv v1[0], 0x3F0(zero)")]
        [InlineData(0xE801_2040u, "sqv v1[0], -0x400(zero)")]
        public void An_rsp_vector_transfer_prints_its_scaled_byte_offset(uint word, string expected)
        {
            Assert.Equal(expected, Rsp(word));
        }

        // A DMEM address is twelve bits, so a negative offset from zero names the top of DMEM - see §4.
        [Theory]
        [InlineData(0x8C04_0010u, StaticReferenceKind.Read, 0x010u)]
        [InlineData(0xAC04_FFFCu, StaticReferenceKind.Write, 0xFFCu)]
        [InlineData(0x9C04_0010u, StaticReferenceKind.Read, 0x010u)]
        [InlineData(0xC802_07C0u, StaticReferenceKind.Read, 0xFC0u)]
        [InlineData(0xC806_127Fu, StaticReferenceKind.Read, 0xFFCu)]
        [InlineData(0xE801_203Fu, StaticReferenceKind.Write, 0x3F0u)]
        [InlineData(0xE801_2040u, StaticReferenceKind.Write, 0xC00u)]
        [InlineData(0xE801_5001u, StaticReferenceKind.Write, 0x010u)]
        public void An_rsp_access_based_on_zero_names_its_dmem_address(uint word, StaticReferenceKind kind, uint target)
        {
            var instruction = RspDisassembler.Decode(word, 0);

            Assert.Equal(kind, instruction.Kind);
            Assert.Equal(target, instruction.Target);
        }

        [Theory]
        [InlineData(0xC881_2001u)]
        [InlineData(0x8C85_0FFCu)]
        [InlineData(0x4008_2000u)]
        [InlineData(0x4A03_1040u)]
        [InlineData(0x03E0_0008u)]
        public void An_rsp_instruction_with_no_constant_address_names_nothing(uint word)
        {
            var instruction = RspDisassembler.Decode(word, 0);

            Assert.Null(instruction.Kind);
            Assert.Equal(0u, instruction.Target);
        }

        // The load format 10 does nothing; the rest are encodings the processor does not own - see §3.1.
        [Theory]
        [InlineData(0xC88C_5001u)]
        [InlineData(0xC880_6001u)]
        [InlineData(0xE801_6001u)]
        [InlineData(0x0482_0004u)]
        [InlineData(0x0493_0004u)]
        [InlineData(0x0000_000Cu)]
        [InlineData(0x0085_0018u)]
        [InlineData(0x0085_102Du)]
        [InlineData(0x0000_0038u)]
        [InlineData(0x4028_2000u)]
        [InlineData(0x4868_0000u)]
        [InlineData(0x4A03_105Eu)]
        [InlineData(0x4A03_106Fu)]
        [InlineData(0x4A03_107Bu)]
        [InlineData(0x4400_0000u)]
        [InlineData(0x5085_0001u)]
        [InlineData(0xDC04_0010u)]
        [InlineData(0xC404_0010u)]
        [InlineData(0xBC19_0010u)]
        public void An_rsp_encoding_the_processor_does_not_own_is_a_word(uint word)
        {
            var instruction = RspDisassembler.Decode(word, 0);

            Assert.Equal(".word", instruction.Mnemonic);
            Assert.Equal($"0x{word:X8}", instruction.Operands);
            Assert.Null(instruction.Kind);
        }

        // Neither decoder may throw on any word; a sweep of every opcode, function and sub-opcode - see §3.
        [Fact]
        public void No_word_makes_either_decoder_throw()
        {
            uint[] fills = { 0x0000_0000, 0x03FF_FFC0, 0x0155_5540, 0x02AA_AA80, 0x03E0_F800, 0x001F_07C0 };
            int decoded = 0;

            for (uint op = 0; op < 64; op++)
            {
                for (uint sub = 0; sub < 32; sub++)
                {
                    for (uint low = 0; low < 64; low++)
                    {
                        foreach (uint fill in fills)
                        {
                            uint word = (op << 26) | (sub << 21) | (sub << 16) | (fill & 0x001F_FFC0 & ~(0x1Fu << 16)) | low;

                            Assert.False(string.IsNullOrEmpty(Vr4300Disassembler.Decode(word, 0x8000_0000 + low * 4).Mnemonic));
                            Assert.False(string.IsNullOrEmpty(RspDisassembler.Decode(word, low * 4).Mnemonic));
                            decoded++;
                        }
                    }
                }
            }

            Assert.Equal(64 * 32 * 64 * 6, decoded);
        }
    }
}
