using System;
using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Process-wide cwd, walled to the sandbox - see `man cd` and `man hier`.
    public class CdCommand : IDianaOSCommand
    {
        private readonly Func<DianaOSInterpreter> _self;

        public CdCommand(Func<DianaOSInterpreter> self)
        {
            _self = self;
        }

        public string Name => "cd";
        public bool IsReadOnly => false;
        public string Usage => "  cd [dir]                      change directory, walled to the project root (no arg: your home directory)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            // Joins every word (see `man cd`); no argument means '/', which is home (see `man hier`).
            string dir = args.Length >= 2 ? string.Join(' ', args.Skip(1)) : "/";

            if (!DianaOSSandbox.TryResolve(dir, out string resolved))
            {
                return DianaOSResult.Fail($"cd: '{dir}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            if (!Directory.Exists(resolved))
            {
                return DianaOSResult.Fail($"cd: no such directory: {dir}");
            }

            try
            {
                Environment.CurrentDirectory = resolved;
            }
            catch (Exception ex)
            {
                return DianaOSResult.Fail($"cd: {ex.Message}");
            }

            return DianaOSResult.Ok("");
        }
    }
}
