namespace EmuSen.Debug.Commands
{
    // One command's complete implementation - the name it responds to,
    // its own usage text, and the logic itself, in one small class.
    // DebugCommandProcessor.cs's own header comment already describes
    // this shape for the *commands themselves* ("small and single-
    // purpose... the same 'small composable tools' shape as a Linux
    // toolchain... rather than a monolith") - this interface applies
    // that same idea to the *implementation*, so the processor itself
    // doesn't have to grow into the monolith it was built to avoid.
    //
    // Stateful commands (search, snapshot) just hold their session state
    // as private instance fields, same as they did as private fields on
    // DebugCommandProcessor before this split - DebugCommandProcessor
    // constructs each IDebugCommand exactly once and reuses that same
    // instance for every call, so nothing about moving the state changes
    // when it's created or how long it lives.
    public interface IDebugCommand
    {
        // The word typed at the F4 prompt to reach this command (e.g.
        // "mem", "search") - matched case-insensitively by whoever owns
        // the command registry.
        string Name { get; }

        // Help text for this command, already formatted to match the
        // rest of `help`'s output (two-space indent, aligned columns) -
        // can be multiple lines (embedded '\n') for a command with
        // several sub-verbs, the way `watch`/`search`/`snapshot` already
        // are.
        string Usage { get; }

        // parts[0] is always this command's own Name (already consumed
        // by dispatch) - parts[1..] are the actual arguments. Exceptions
        // propagate to the caller, which is expected to catch them and
        // turn them into a readable "Error: ..." string rather than
        // crashing whatever's running the prompt - same contract
        // DebugCommandProcessor.Execute already had.
        string Execute(IDebugTarget target, string[] parts);
    }
}
