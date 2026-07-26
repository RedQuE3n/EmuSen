namespace EmuSen.Shell.Commands
{
    // Unix `pwd` - prints the process's current working directory, the
    // same one `cd` changes and every relative path this shell touches
    // (redirection, `dump`/`load`, `mv`, `ls`, `wc <path>`, ...) resolves
    // against, since none of them track a shell-private cwd separate from
    // Environment.CurrentDirectory.
    public class PwdCommand : IShellCommand
    {
        public string Name => "pwd";
        public string Usage => "  pwd                           print the current working directory";

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin) =>
            System.Environment.CurrentDirectory;
    }
}
