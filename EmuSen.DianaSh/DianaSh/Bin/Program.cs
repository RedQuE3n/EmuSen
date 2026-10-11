using System;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin
{
    // Standalone entry point - see `man tmux` for the session model.
    class Program
    {
        static void Main(string[] args)
        {
            var sessions = new DianaShSessionManager();
            DianaShInterpreter first = DianaShInterpreter.CreateDefault(null, sessions: sessions);
            sessions.RegisterInitial(new DianaShSession("main", first));

            Console.WriteLine(first.GetWelcomeBanner(new[] { "(none - launched standalone, no core loaded)" }));
            Console.WriteLine("--- DianaSh standalone shell - type 'help' or 'man <command>', 'shutdown' to quit ---");

            while (true)
            {
                DianaShInterpreter shell = sessions.Current!.Interpreter;
                string prompt = sessions.Sessions.Count > 1 ? $"DianaSh [{sessions.Current!.Name}] #: " : "DianaSh #: ";
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
