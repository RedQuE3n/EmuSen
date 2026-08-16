using System;
using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // An existing destination directory is moved into, not replaced - see `man mv`.
    public class MvCommand : IDianaOSCommand
    {
        public string Name => "mv";
        public bool IsReadOnly => false;
        public string Usage => "  mv <src> <dst>                SUSPENDED - move/rename a file or directory (dst may be an existing directory)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (DianaOSCommandSuspensions.IsSuspended(Name)) return DianaOSCommandSuspensions.Refuse(Name);

            if (args.Length < 3) return DianaOSResult.Fail("mv: usage: mv <src> <dst>");

            if (!DianaOSSandbox.TryResolve(args[1], out string src))
            {
                return DianaOSResult.Fail($"mv: '{args[1]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }
            if (!DianaOSSandbox.TryResolve(args[2], out string dst))
            {
                return DianaOSResult.Fail($"mv: '{args[2]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            if (Directory.Exists(dst)) dst = Path.Combine(dst, Path.GetFileName(src.TrimEnd('/', '\\')));

            try
            {
                if (Directory.Exists(src))
                {
                    if (File.Exists(dst) || Directory.Exists(dst))
                    {
                        return DianaOSResult.Fail($"mv: destination already exists: {dst}");
                    }
                    Directory.Move(src, dst);
                }
                else if (File.Exists(src))
                {
                    File.Move(src, dst, overwrite: true);
                }
                else
                {
                    return DianaOSResult.Fail($"mv: no such file or directory: {src}");
                }
            }
            catch (Exception ex)
            {
                return DianaOSResult.Fail($"mv: {ex.Message}");
            }

            return DianaOSResult.Ok("");
        }
    }
}
