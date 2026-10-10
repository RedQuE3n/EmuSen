using System;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Acts on the concrete core, so it takes delegates rather than a target - see §3.3b.
    public class StateCommand : IDianaShCommand
    {
        private readonly Action<string> _save;
        private readonly Action<string> _load;
        private readonly Func<string> _defaultPath;

        public StateCommand(Action<string> save, Action<string> load, Func<string> defaultPath)
        {
            _save = save;
            _load = load;
            _defaultPath = defaultPath;
        }

        public string Name => "state";
        public bool IsReadOnly => false;
        public string Usage => "  state save|load [path]        save/load emulator state (defaults to this frontend's own save-state path if omitted)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2)
            {
                return DianaShResult.Fail("Usage: state save|load [path]  (defaults to the F5/F9 path if omitted)");
            }

            string sub = args[1].ToLowerInvariant();
            if (sub is not ("save" or "load"))
            {
                return DianaShResult.Fail("Usage: state save|load [path]  (defaults to the F5/F9 path if omitted)");
            }

            // A typed path is the sandbox's; the host's own default is used as the host gives it - see `man state`.
            string path = args.Length >= 3 ? args[2] : _defaultPath();
            string resolved = path;
            if (args.Length >= 3 && !DianaShSandbox.TryResolve(path, out resolved))
            {
                return DianaShResult.Fail($"state: '{path}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
            }

            if (sub == "save")
            {
                try
                {
                    _save(resolved);
                    return DianaShResult.Ok($"[STATE] Saved: {path}");
                }
                catch (Exception ex)
                {
                    return DianaShResult.Fail($"[STATE] Save failed: {ex.Message}");
                }
            }

            if (sub == "load")
            {
                try
                {
                    _load(resolved);
                    return DianaShResult.Ok($"[STATE] Loaded: {path}");
                }
                catch (Exception ex)
                {
                    return DianaShResult.Fail($"[STATE] Load failed: {ex.Message}");
                }
            }

            return DianaShResult.Fail("Usage: state save|load [path]  (defaults to the F5/F9 path if omitted)");
        }
    }
}
