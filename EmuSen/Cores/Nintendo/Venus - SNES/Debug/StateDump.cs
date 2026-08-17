using System.Text;
using EmuSen.Cores.Nintendo.Venus.Processor;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Video;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // On-demand snapshot of CPU + PPU state, formatted to be directly comparable to MesenCE's own Status.
    public static class StateDump
    {
        public static string DumpCpuState(Cpu cpu)
        {
            var sb = new StringBuilder();
            sb.Append("[STATE] CPU  ");
            sb.Append($"PC={cpu.PB:X2}:{cpu.PC:X4}  ");
            sb.Append($"A={cpu.A:X4} X={cpu.X:X4} Y={cpu.Y:X4} S={cpu.S:X4} D={cpu.D:X4} DB={cpu.DB:X2}  ");
            sb.Append($"P={cpu.P:X2} [{DescribeFlags(cpu.P, cpu.E)}]  ");
            sb.Append($"E={(cpu.E ? 1 : 0)}");
            return sb.ToString();
        }

        // P's bit meanings differ slightly between native (E=0) and emulation (E=1) mode - bit 4 is X (index.
        private static string DescribeFlags(byte p, bool emulation)
        {
            var sb = new StringBuilder();
            sb.Append((p & 0x80) != 0 ? 'N' : 'n');
            sb.Append((p & 0x40) != 0 ? 'V' : 'v');
            if (emulation)
            {
                sb.Append('-');
                sb.Append((p & 0x10) != 0 ? 'B' : 'b');
            }
            else
            {
                sb.Append((p & 0x20) != 0 ? 'M' : 'm');
                sb.Append((p & 0x10) != 0 ? 'X' : 'x');
            }
            sb.Append((p & 0x08) != 0 ? 'D' : 'd');
            sb.Append((p & 0x04) != 0 ? 'I' : 'i');
            sb.Append((p & 0x02) != 0 ? 'Z' : 'z');
            sb.Append((p & 0x01) != 0 ? 'C' : 'c');
            return sb.ToString();
        }

        public static string DumpPpuState(Ppu ppu)
        {
            var sb = new StringBuilder();
            sb.AppendLine($"[STATE] PPU  BGMODE={ppu.Bgmode:X2}  INIDISP={ppu.Inidisp:X2}  TM={ppu.Tm:X2}  TS={ppu.Ts:X2}");
            sb.AppendLine($"[STATE] PPU  CGWSEL={ppu.Cgwsel:X2}  CGADSUB={ppu.Cgadsub:X2}  MOSAIC={ppu.Mosaic:X2} (anchor={ppu.MosaicStartScanline})  SETINI={ppu.Setini:X2}");
            sb.AppendLine($"[STATE] PPU  BG1 SC={ppu.BgSc[0]:X2} X={ppu.BgScrollX[0]} Y={ppu.BgScrollY[0]}   BG2 SC={ppu.BgSc[1]:X2} X={ppu.BgScrollX[1]} Y={ppu.BgScrollY[1]}");
            sb.AppendLine($"[STATE] PPU  BG3 SC={ppu.BgSc[2]:X2} X={ppu.BgScrollX[2]} Y={ppu.BgScrollY[2]}   BG4 SC={ppu.BgSc[3]:X2} X={ppu.BgScrollX[3]} Y={ppu.BgScrollY[3]}");
            sb.AppendLine($"[STATE] PPU  BG12NBA={ppu.Bg12Nba:X2}  BG34NBA={ppu.Bg34Nba:X2}  OBSEL={ppu.Obsel:X2}");
            sb.Append($"[STATE] PPU  M7SEL={ppu.M7Sel:X2}  M7A={ppu.M7A} M7B={ppu.M7B} M7C={ppu.M7C} M7D={ppu.M7D}  M7X={ppu.M7X} M7Y={ppu.M7Y}  M7HOFS={ppu.M7HOfs} M7VOFS={ppu.M7VOfs}");
            return sb.ToString();
        }

        // Convenience wrapper for the common case - both dumps, one call.
        public static string DumpAll(Cpu cpu, MemoryBus bus)
        {
            return DumpCpuState(cpu) + "\n" + DumpPpuState(bus.Ppu);
        }
    }
}
