using System;
using System.IO;

namespace EmuSen.DianaOS.Commands
{
    // Unix `clear` - wipes whatever's currently on screen so the next
    // prompt starts at the top instead of at the bottom of a long
    // scrollback. Genuinely a real terminal operation (`Console.Clear()`),
    // not just printing blank lines, so it only does anything useful
    // against a real interactive console - same requirement `nano`/
    // `coretop` already have, just without needing to actually refuse
    // when it isn't met: a redirected/nonexistent console throws
    // IOException here rather than silently doing nothing, so that's
    // caught and swallowed instead of surfacing as a shell error - a
    // piped script or a headless test running `clear` shouldn't fail
    // over a screen that was never going to be looked at anyway.
    //
    // `EmuSen.Mistress9`'s own DianaOS console window isn't a real
    // terminal at all (a TextBox, same reason `coretop -w`/`feed`
    // needed frontend-specific replacements) - see
    // DianaOSConsoleWindow.axaml.cs's own `clear` override, which
    // replaces this default via the same extraCommands override-by-name
    // mechanism `coretop`/`pause`/`resume`/`feed` already use there.
    public class ClearCommand : IDianaOSCommand
    {
        public string Name => "clear";
        public string Usage => "  clear                         clear the terminal screen";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            try { Console.Clear(); }
            catch (IOException) { }
            return DianaOSResult.Ok(string.Empty);
        }
    }
}
