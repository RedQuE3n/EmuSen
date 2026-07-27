using System;
using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    public class RegsCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "regs";
        public bool IsReadOnly => true;
        public string Usage => "  regs                          CPU + video registers";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            var sb = new StringBuilder();
            sb.AppendLine($"{target.CoreName} CPU registers:");
            foreach (var r in target.CpuRegisters.Current)
            {
                int digits = Math.Max(1, r.BitWidth / 4);
                sb.AppendLine($"  {r.Name,-4} = 0x{r.Value.ToString("X" + digits)}");
            }
            sb.AppendLine("Video registers:");
            foreach (var r in target.VideoRegisters.Current)
            {
                sb.AppendLine($"  {r.Name,-8} = 0x{r.Value:X2}");
            }
            var apuRegs = target.ApuRegisters.Current;
            if (apuRegs.Count > 0)
            {
                sb.AppendLine("APU registers:");
                foreach (var r in apuRegs)
                {
                    int digits = Math.Max(1, r.BitWidth / 4);
                    sb.AppendLine($"  {r.Name,-9} = 0x{r.Value.ToString("X" + digits)}");
                }
            }
            return sb.ToString().TrimEnd();
        }
    }
}
