using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Unix `pwd` - prints the process's current working directory, the
    // same one `cd` changes and every relative path this shell touches
    // (redirection, `dump`/`load`, `mv`, `ls`, `wc <path>`, ...) resolves
    // against, since none of them track a shell-private cwd separate from
    // Environment.CurrentDirectory.
    public class PwdCommand : IDianaOSCommand
    {
        public string Name => "pwd";
        public bool IsReadOnly => true;
        public string Usage => "  pwd                           print the current working directory";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) =>
            System.Environment.CurrentDirectory;
    }
}
