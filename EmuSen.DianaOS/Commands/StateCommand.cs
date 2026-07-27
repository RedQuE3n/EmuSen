using System;

namespace EmuSen.DianaOS.Commands
{
    // Save/load emulator state - promoted from being a hand-rolled string
    // match inside EmuSen.Hotaru's own RunDebugPrompt (and, before this,
    // never reachable from EmuSen.Mistress9's console at all, only its
    // Save/Load State menu items) into a real, shared IDianaOSCommand, so
    // DianaOS's own registry is genuinely "everything you can do" rather
    // than "everything except this."
    //
    // Doesn't touch IDebugTarget at all - a save/load state operation acts
    // on the concrete ICore a frontend is driving, not on the core-
    // agnostic debug-inspection surface IDebugTarget models, so this takes
    // save/load/defaultPath as constructor-injected delegates instead (the
    // same Mechanism-A shape PauseCommand/CoretopCommand already use for
    // "this needs to reach outside IDebugTarget entirely"). Each frontend
    // wires its own ICore.SaveState/LoadState (or equivalent session
    // wrapper) directly - no frontend-specific subclass needed.
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
