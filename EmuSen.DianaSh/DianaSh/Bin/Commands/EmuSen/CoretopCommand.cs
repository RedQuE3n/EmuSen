using System;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // A front end's `coretop`: the live hardware dashboard in a window of its own over the front end's session, in place of the shell's terminal one - see `man coretop` and EmuSen_Backend.md §40.10.
    public class CoretopCommand : IDianaShCommand
    {
        private readonly Action? _openWindow;
        private readonly Func<bool> _gameRunning;

        // Lets a frontend wire the window without DianaSh knowing any toolkit exists - see §3.3b.
        public CoretopCommand(Func<bool> gameRunning) => _gameRunning = gameRunning;

        public CoretopCommand(Action openWindow, Func<bool> gameRunning)
        {
            _openWindow = openWindow;
            _gameRunning = gameRunning;
        }

        public string Name => "coretop";
        public bool IsReadOnly => false;
        public string Usage => "  coretop [-w]                  live dashboard of the running game's hardware, in a window of this frontend's\n" +
                                "                                (-w, the window, is all a frontend's own coretop opens)";

        public DianaShResult Execute(string[] args, string? stdin)
        {
            if (!_gameRunning()) return DianaShResult.Fail("No ROM loaded - this command needs an active debug target.");
            if (_openWindow is null) return DianaShResult.Fail("coretop: -w is not supported by this frontend.");
            _openWindow();
            return DianaShResult.Ok("coretop: opened in a separate window.");
        }
    }
}
