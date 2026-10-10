using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // An existing destination directory is moved into, not replaced - see `man mv`.
    public class MvCommand : IDianaShCommand
    {
        public string Name => "mv";
        public bool IsReadOnly => false;
        public string Usage => "  mv <src> <dst>                SUSPENDED - move/rename a file or directory (dst may be an existing directory)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (DianaShCommandSuspensions.IsSuspended(Name)) return DianaShCommandSuspensions.Refuse(Name);

            if (args.Length < 3) return DianaShResult.Fail("mv: usage: mv <src> <dst>");

            if (!DianaShSandbox.TryResolve(args[1], out string src))
            {
                return DianaShResult.Fail($"mv: '{args[1]}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
            }
            if (!DianaShSandbox.TryResolve(args[2], out string dst))
            {
                return DianaShResult.Fail($"mv: '{args[2]}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
            }

            if (Directory.Exists(dst)) dst = Path.Combine(dst, Path.GetFileName(src.TrimEnd('/', '\\')));

            try
            {
                if (Directory.Exists(src))
                {
                    if (File.Exists(dst) || Directory.Exists(dst))
                    {
                        return DianaShResult.Fail($"mv: destination already exists: {dst}");
                    }
                    Directory.Move(src, dst);
                }
                else if (File.Exists(src))
                {
                    File.Move(src, dst, overwrite: true);
                }
                else
                {
                    return DianaShResult.Fail($"mv: no such file or directory: {src}");
                }
            }
            catch (Exception ex)
            {
                return DianaShResult.Fail($"mv: {ex.Message}");
            }

            return DianaShResult.Ok("");
        }
    }
}
