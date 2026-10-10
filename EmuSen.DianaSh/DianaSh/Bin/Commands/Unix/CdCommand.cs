using System;
using System.IO;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Process-wide cwd, walled to the sandbox - see `man cd` and `man hier`.
    public class CdCommand : IDianaShCommand
    {
        private readonly Func<DianaShInterpreter> _self;

        public CdCommand(Func<DianaShInterpreter> self)
        {
            _self = self;
        }

        public string Name => "cd";
        public bool IsReadOnly => false;
        public string Usage => "  cd [dir]                      change directory, walled to the project root (no arg: your home directory)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            // Joins every word (see `man cd`); no argument means '/', which is home (see `man hier`).
            string dir = args.Length >= 2 ? string.Join(' ', args.Skip(1)) : "/";

            if (!DianaShSandbox.TryResolve(dir, out string resolved))
            {
                return DianaShResult.Fail($"cd: '{dir}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
            }

            if (!Directory.Exists(resolved))
            {
                return DianaShResult.Fail($"cd: no such directory: {dir}");
            }

            try
            {
                Environment.CurrentDirectory = resolved;
            }
            catch (Exception ex)
            {
                return DianaShResult.Fail($"cd: {ex.Message}");
            }

            return DianaShResult.Ok("");
        }
    }
}
