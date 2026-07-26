using System.Text;

namespace EmuSen.Shell.Commands
{
    public class SpacesCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "spaces";
        public string Usage => "  spaces                        list available memory spaces";

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.Shell.Commands.DebugCommandHelpers.RequireTarget(target);
            var sb = new StringBuilder();
            sb.AppendLine($"{target.CoreName} memory spaces:");
            foreach (var s in target.GetMemorySpaces())
            {
                sb.AppendLine($"  {s.Name,-10} {s.Size,8} bytes  {(s.IsWritable ? "R/W" : "R/O")}");
            }
            return sb.ToString().TrimEnd();
        }
    }
}
