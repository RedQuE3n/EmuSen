using System.Text;
using EmuSen.Cores.Nintendo.Venus.Processor;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Video;
using EmuSen.Crystal.Systems.Snes;
using EmuSen.Crystal.Systems;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // On-demand snapshot of CPU + PPU state, formatted to be directly comparable to MesenCE's own Status.
    public static class StateDump
    {
        // The 65816's line, written by the SNES's system pack so VenusRT's summary prints the same one.
        public static string DumpCpuState(Cpu cpu) =>
            SnesDebugViews.CpuStateLine(cpu.PB, cpu.PC, cpu.A, cpu.X, cpu.Y, cpu.S, cpu.D, cpu.DB, cpu.P, cpu.E);

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
