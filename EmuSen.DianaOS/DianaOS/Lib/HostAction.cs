using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Lib
{
    // A signal a command needs to hand back to whatever's driving this
    // interpreter's own loop (a frontend's RunDebugPrompt, or Main itself)
    // rather than just producing text - the escape hatch DianaOSResult
    // never had before this. Reserved strictly for cases where a command
    // needs to change the CALLER's control flow (break a loop, hand a
    // value back up several stack frames): resuming/shutting down/single-
    // stepping/swapping the loaded core. Side-effect-only needs (pausing a
    // thread, opening a window, editing a text file) stay on the already-
    // proven Mechanism-A pattern instead (a constructor-injected
    // Action/Func<T> delegate, same shape as PauseCommand/CoretopCommand)
    // and never touch this type at all - see DianaOSResult's own comment
    // on why this field is optional, not a replacement for that pattern.
    //
    // A closed abstract record hierarchy, not an enum+object payload -
    // matches this codebase's existing taste for small, explicit,
    // strongly-typed result shapes (DebugLoadInfo, StaticReferenceKind)
    // rather than a stringly/object-typed grab bag, and gives LoadCore's
    // two strings real type safety at every consumption site instead of
    // a cast.
    public abstract record HostAction
    {
        private HostAction() { }

        // 'shutdown'/'quit' - terminate the whole process.
        public sealed record Shutdown : HostAction;

        // 'resume'/'continue'/'c' - stop blocking on the interactive
        // prompt and let the emulation loop keep running.
        public sealed record Resume : HostAction;

        // 'step'/'s' - single-step one instruction, then return to the
        // same "keep going" state Resume produces (BreakpointRegistry's
        // own ArmSingleStep already does the actual arming; this just
        // carries the "now let RunFrame() run again" signal alongside it).
        public sealed record Step : HostAction;

        // 'core <corename> <path>' - swap the loaded ROM in place. Two
        // plain strings, not a resolved object, since resolving/validating
        // them (unknown core name, missing file, wrong extension) is the
        // issuing command's own job - by the time this reaches whatever's
        // driving the loop, both are already known-good.
        public sealed record LoadCore(string CoreName, string RomPath) : HostAction;

        // 'tmux new/switch/kill' - see `man tmux`.
        public sealed record SwitchSession(string Name) : HostAction;
    }
}
