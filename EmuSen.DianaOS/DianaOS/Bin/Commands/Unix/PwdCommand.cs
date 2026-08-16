using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // The process cwd, since nothing here tracks a shell-private one - see `man pwd`.
    public class PwdCommand : IDianaOSCommand
    {
        public string Name => "pwd";
        public bool IsReadOnly => true;
        public string Usage => "  pwd                           print the current working directory";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) =>
            System.Environment.CurrentDirectory;
    }
}
