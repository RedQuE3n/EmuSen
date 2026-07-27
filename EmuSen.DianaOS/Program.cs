using System;
using EmuSen.DianaOS;

namespace EmuSen.DianaOS
{
    // The standalone entry point this project exists to make possible -
    // launches DianaOS as its own real shell process, with no emulator
    // core loaded at all and no host (Hotaru/Mistress9/Pharaoh90) wrapping
    // it. Deliberately as thin as it can be: this project has zero
    // reference to any Cores project (that's the whole point of the
    // decoupling work that preceded this project split - see IDebugTarget's
    // own header comment), so there's no `core <name> <path>` to resolve
    // here the way EmuSen.Hotaru's own RunStandaloneShell has - any command
    // needing a real target just reports so (DebugCommandHelpers.RequireTarget's
    // existing behavior), the same graceful-degradation every core-agnostic
    // command already supports for a null target.
    //
    // Every other project in this solution reaches DianaOSInterpreter the
    // same way this file does (CreateDefault(null) is exactly what
    // EmuSen.Hotaru's own RunStandaloneShell already does before a ROM
    // loads) - this file is just that same call, with nothing else around
    // it.
    class Program
    {
        static void Main(string[] args)
        {
            DianaOSInterpreter shell = DianaOSInterpreter.CreateDefault(null);
            Console.WriteLine(shell.GetWelcomeBanner(new[] { "(none - launched standalone, no core loaded)" }));
            Console.WriteLine("--- DianaOS standalone shell - type 'help' or 'man <command>', 'shutdown' to quit ---");

            while (true)
            {
                Console.Write(shell.IsAwaitingMoreInput ? "> " : "DianaOS #: ");
                string? line = ConsoleLineReader.ReadLine(shell.History.Entries);
                if (line is null) return; // EOF - stdin closed/redirected input exhausted

                (_, string output, HostAction? action) = shell.Submit(line);
                if (output.Length > 0) Console.WriteLine(output);
                if (action is HostAction.Shutdown) return;
            }
        }
    }
}
