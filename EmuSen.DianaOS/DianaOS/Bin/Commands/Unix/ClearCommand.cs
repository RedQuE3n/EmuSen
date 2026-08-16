using System;
using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // A real Console.Clear(); a redirected console throws, so that is swallowed - see §3.17.
    public class ClearCommand : IDianaOSCommand
    {
        public string Name => "clear";
        public bool IsReadOnly => true;
        public string Usage => "  clear                         clear the terminal screen";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            try { Console.Clear(); }
            catch (IOException) { }
            return DianaOSResult.Ok(string.Empty);
        }
    }
}
