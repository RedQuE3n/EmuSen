namespace EmuSen.DianaOS.Commands.Unix
{
    // Terminates the whole process - promoted from a hand-rolled string
    // match in both EmuSen.Hotaru's RunDebugPrompt and its own separate
    // RunStandaloneShell bypass into one real, shared command via
    // HostAction.Shutdown (see that type's own comment). 'quit' is
    // recognized as an alias directly in DianaOSInterpreter.Dispatch
    // (normalized to 'shutdown' before the registry lookup), not a
    // second registry entry, so 'help' doesn't print the same line
    // twice.
    public class ShutdownCommand : IDianaOSCommand
    {
        public string Name => "shutdown";
        public bool IsReadOnly => false;
        public string Usage => "  shutdown | quit               terminate the process";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
            => DianaOSResult.Ok("Shutting down.", new HostAction.Shutdown());
    }
}
