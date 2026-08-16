using System;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin
{
    // Standalone entry point - see `man tmux` for the session model.
    class Program
    {
        static void Main(string[] args)
        {
            var sessions = new DianaOSSessionManager();
            DianaOSInterpreter first = DianaOSInterpreter.CreateDefault(null, sessions: sessions);
            sessions.RegisterInitial(new DianaOSSession("main", first));

            Console.WriteLine(first.GetWelcomeBanner(new[] { "(none - launched standalone, no core loaded)" }));
            Console.WriteLine("--- DianaOS standalone shell - type 'help' or 'man <command>', 'shutdown' to quit ---");

            while (true)
            {
                DianaOSInterpreter shell = sessions.Current!.Interpreter;
                string prompt = sessions.Sessions.Count > 1 ? $"DianaOS [{sessions.Current!.Name}] #: " : "DianaOS #: ";
                Console.Write(shell.IsAwaitingMoreInput ? "> " : prompt);
                string? line = ConsoleLineReader.ReadLine(shell.History.Entries);
                if (line is null) return; // EOF - stdin closed/redirected input exhausted

                (_, string output, HostAction? action) = shell.Submit(line);
                if (output.Length > 0) Console.WriteLine(output);
                if (action is HostAction.Shutdown) return;
            }
        }
    }
}
