
namespace EmuSen.DianaOS
{
    // One command's complete implementation - unifies what used to be two
    // separate interfaces (IDebugCommand, for commands needing real
    // IDebugTarget access, and ITextFilterCommand, for pipe-stage-only
    // text tools like sed) into one. Every command gets both an ambient
    // target reference AND an optional stdin string now - a plain debug
    // command (mem, regs, ...) just ignores stdin, the way real `ls`
    // ignores stdin; a text filter (sed) just ignores target, the way
    // real `sed` doesn't care what process invoked it.
    public interface IDianaOSCommand
    {
        // The word typed to reach this command (e.g. "mem", "sed") -
        // matched case-insensitively by DianaOSInterpreter's registry.
        string Name { get; }

        // True if this command never mutates core/session/interpreter
        // state for ANY invocation of it - only inspects and reports.
        // Drives whether a frontend's live (non-halted) terminal may run
        // this command immediately, off the emulation thread, instead of
        // queuing it for the next frame (see
        // DianaOSInterpreter.TryGetReadOnlyFastPath and
        // EmuSen.Hotaru's GameWindow console reader thread). A command
        // with a mixed read/write sub-verb surface (e.g. `watch add` vs
        // `watch list`) must report false here - this is a per-class
        // property, not something that can vary by args - which only
        // costs its read-only sub-verbs one frame of latency.
        bool IsReadOnly { get; }

        // Help text for this command, formatted to match the rest of
        // `help`'s output (two-space indent, aligned columns) - can be
        // multiple lines for a command with several sub-verbs.
        string Usage { get; }

        // `target` is the attached emulator/debug session, or null if the
        // shell is running with no core loaded yet (or ever - nothing
        // stops a future non-emulator use of this same shell engine).
        // Commands that need real hardware access (mem, regs, watch...)
        // should treat a null target as a normal, expected "no ROM
        // loaded" condition, not throw.
        //
        // `args` is this command's own tokenized, already-expanded
        // argument list - args[0] is always this command's Name, args[1..]
        // are its real arguments (same convention IDebugCommand always
        // used).
        //
        // `stdin` is the previous pipeline stage's output text, or null if
        // this command is running standalone or is the first stage of a
        // pipeline - distinct from an empty string, which means a
        // previous stage genuinely produced no output.
        DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin);
    }
}
