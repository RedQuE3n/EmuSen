namespace EmuSen.Debug.Commands
{
    // Opt-in second interface for a command that can also sit AFTER a `|`
    // in a pipeline (see DebugCommandProcessor.Execute's pipe-splitting
    // logic) - most commands (mem, regs, sprites...) read from the live
    // IDebugTarget and wouldn't mean anything fed a plain text string
    // instead, so this is deliberately separate from IDebugCommand rather
    // than a new required member every command has to implement.
    //
    // A command implementing both interfaces (SedCommand, EchoCommand) is
    // usable two ways: standalone, via IDebugCommand.Execute exactly like
    // any other command, or as a pipeline stage, via this Filter method -
    // same "small composable tools" idea IDebugCommand's own comment
    // describes, just extended to actually let one tool's output feed
    // another's input the way a real Unix shell does.
    public interface ITextFilterCommand
    {
        // `input` is whatever the previous pipeline stage produced (the
        // first stage's own Execute() return value, or an earlier
        // filter's Filter() return value). `parts` is this stage's own
        // tokenized command line, same convention as IDebugCommand.Execute
        // (parts[0] is this command's Name, parts[1..] are its arguments).
        string Filter(IDebugTarget target, string input, string[] parts);
    }
}
