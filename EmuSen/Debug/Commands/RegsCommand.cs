using System;
using System.Text;

namespace EmuSen.Debug.Commands
{
    public class RegsCommand : IDebugCommand
    {
        public string Name => "regs";
        public string Usage => "  regs                          CPU + video registers";

        public string Execute(IDebugTarget target, string[] parts)
        {
            var sb = new StringBuilder();
            sb.AppendLine($"{target.CoreName} CPU registers:");
            foreach (var r in target.GetCpuRegisters())
            {
                int digits = Math.Max(1, r.BitWidth / 4);
                sb.AppendLine($"  {r.Name,-4} = 0x{r.Value.ToString("X" + digits)}");
            }
            sb.AppendLine("Video registers:");
            foreach (var r in target.GetVideoRegisters())
            {
                sb.AppendLine($"  {r.Name,-8} = 0x{r.Value:X2}");
            }
            var apuRegs = target.GetApuRegisters();
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
