using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // The process cwd, since nothing here tracks a shell-private one - see `man pwd`.
    public class PwdCommand : IDianaShCommand
    {
        public string Name => "pwd";
        public bool IsReadOnly => true;
        public string Usage => "  pwd                           print the current working directory";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin) =>
            System.Environment.CurrentDirectory;
    }
}
