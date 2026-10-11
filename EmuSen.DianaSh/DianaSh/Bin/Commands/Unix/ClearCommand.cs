using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // A real Console.Clear(); a redirected console throws, so that is swallowed - see §3.17.
    public class ClearCommand : IDianaShCommand
    {
        public string Name => "clear";
        public bool IsReadOnly => true;
        public string Usage => "  clear                         clear the terminal screen";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            try { Console.Clear(); }
            catch (IOException) { }
            return DianaShResult.Ok(string.Empty);
        }
    }
}
