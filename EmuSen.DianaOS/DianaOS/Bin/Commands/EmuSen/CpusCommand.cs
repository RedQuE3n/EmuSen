using System.Linq;
using System.Text;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Lists the chips every scoped command can be aimed at - see `man cpus`.
    public class CpusCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "cpus";
        public bool IsReadOnly => true;
        public string Usage =>
            "  cpus                          list the processors a debug command can be scoped to";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = DebugCommandHelpers.RequireTarget(target);

            var cpus = target.DebugCpus;
            if (cpus.Count == 0)
            {
                return $"{target.CoreName} publishes no per-CPU debug list; every command targets its main CPU.";
            }

            var sb = new StringBuilder();
            sb.AppendLine($"{target.CoreName} debug processors:");

            foreach (var cpu in cpus)
            {
                string pc = cpu.ProgramCounter is { } read ? $"${read():X6}" : "-";
                var features = new[]
                {
                    cpu.CanHalt ? "bp" : null,
                    cpu.Coverage != null ? "cov" : null,
                    cpu.CallStack != null ? "bt" : null,
                    cpu.Registers != null ? "regs" : null,
                    cpu.RegisterWriter != null ? "setreg" : null,
                    cpu.CodeSpace != null ? "disasm" : null,
                }.Where(f => f != null);

                sb.AppendLine($"  {cpu.Name,-5} PC={pc,-8} {cpu.Description}");
                sb.AppendLine($"        supports: {string.Join(", ", features)}"
                    + (cpu.CodeSpace != null ? $"   code space: {cpu.CodeSpace}" : ""));
            }

            sb.Append("Scope a command by putting the name first, e.g. `bp sa1 add 8000` or `cov gsu on`.");
            return sb.ToString();
        }
    }
}
