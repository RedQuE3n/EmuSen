using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Resumes emulation after an interactive halt (F4, a breakpoint, a
    // single-step) - promoted from a hand-rolled string match inside
    // EmuSen.Hotaru's own RunDebugPrompt into a real, shared command via
    // HostAction.Resume (see that type's own comment on why this needed
    // a real escape hatch: it has to break the CALLER's own loop, which a
    // plain string result could never express). 'c' is recognized as an
    // alias directly in DianaOSInterpreter.Dispatch (normalized to
    // 'resume' before the registry lookup, the same way 'source'/'.' are
    // already handled), not a separate registry entry. 'continue' is
    // DIFFERENT - it's a real loop-control keyword the parser recognizes
    // before Dispatch ever runs, so a bare (no enclosing loop) 'continue'
    // never reaches this class at all; DianaOSInterpreter's own
    // ContinueSignal catch in SubmitCore sets HostAction.Resume directly
    // instead, matching this shell's own established convention that
    // 'continue' at an interactive prompt means "resume gameplay" - see
    // that catch block's own comment.
    //
    // EmuSen.Mistress9 registers its OWN 'resume' (Views/
    // EmulationControlCommands.cs's ResumeCommand, pause/resume-aware)
    // via extraCommands' override-by-name mechanism, replacing this
    // default entirely - the two frontends have different threading
    // models (see EmuSen_Frontend_Driver.md's own comment on why Hotaru
    // needs no pause/resume signal at all), so this default is exactly
    // right for Hotaru and correctly never reached in Mistress9.
    public class ResumeCommand : IDianaOSCommand
    {
        public string Name => "resume";
        public bool IsReadOnly => false;
        public string Usage => "  resume | continue | c         resume emulation after an interactive halt (F4/breakpoint/single-step)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
            => DianaOSResult.Ok("Resuming.", new HostAction.Resume());
    }
}
