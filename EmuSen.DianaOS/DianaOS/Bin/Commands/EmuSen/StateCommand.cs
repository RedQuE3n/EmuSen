using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Acts on the concrete core, so it takes delegates rather than a target - see §3.3b.
    public class StateCommand : IDianaOSCommand
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

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2)
            {
                return DianaOSResult.Fail("Usage: state save|load [path]  (defaults to the F5/F9 path if omitted)");
            }

            string sub = args[1].ToLowerInvariant();
            string path = args.Length >= 3 ? args[2] : _defaultPath();

            if (sub == "save")
            {
                try
                {
                    _save(path);
                    return DianaOSResult.Ok($"[STATE] Saved: {path}");
                }
                catch (Exception ex)
                {
                    return DianaOSResult.Fail($"[STATE] Save failed: {ex.Message}");
                }
            }

            if (sub == "load")
            {
                try
                {
                    _load(path);
                    return DianaOSResult.Ok($"[STATE] Loaded: {path}");
                }
                catch (Exception ex)
                {
                    return DianaOSResult.Fail($"[STATE] Load failed: {ex.Message}");
                }
            }

            return DianaOSResult.Fail("Usage: state save|load [path]  (defaults to the F5/F9 path if omitted)");
        }
    }
}
