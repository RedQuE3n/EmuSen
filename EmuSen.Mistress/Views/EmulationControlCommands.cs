using System;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.Mistress.Views
{
    // Mistress-only: holds the session through its run control, as the window's own Pause does - see `man pause`.
    public class PauseCommand : HostCommand
    {
        private readonly Action _pause;
        private readonly Func<bool> _isPaused;

        public PauseCommand(Action pause, Func<bool> isPaused)
        {
            _pause = pause;
            _isPaused = isPaused;
        }

        public override string Name => "pause";
        public override bool IsReadOnly => false;
        public override string Usage => "  pause                         pause the emulation thread (safe to read/write emulator state while paused)";

        protected override DianaShResult Run(string[] args, string? stdin)
        {
            if (_isPaused()) return DianaShResult.Ok("Already paused.");
            _pause();
            return DianaShResult.Ok("Emulation paused.");
        }
    }

    public class ResumeCommand : HostCommand
    {
        private readonly Action _resume;
        private readonly Func<bool> _isPaused;

        public ResumeCommand(Action resume, Func<bool> isPaused)
        {
            _resume = resume;
            _isPaused = isPaused;
        }

        public override string Name => "resume";
        public override bool IsReadOnly => false;
        public override string Usage => "  resume                        resume the emulation thread after a pause";

        protected override DianaShResult Run(string[] args, string? stdin)
        {
            if (!_isPaused()) return DianaShResult.Ok("Not paused.");
            _resume();
            return DianaShResult.Ok("Emulation resumed.");
        }
    }

    // Replaces the raw-terminal coretop, which cannot work in a TextBox; the window reads the running session - see `man coretop`.
    public class CoretopWindowCommand : HostCommand
    {
        private readonly Action _openWindow;
        private readonly Func<bool> _gameRunning;

        public CoretopWindowCommand(Action openWindow, Func<bool> gameRunning)
        {
            _openWindow = openWindow;
            _gameRunning = gameRunning;
        }

        public override string Name => "coretop";
        public override bool IsReadOnly => true;
        public override string Usage => "  coretop                       open a live hardware dashboard window (non-blocking - gameplay keeps running)";

        protected override DianaShResult Run(string[] args, string? stdin)
        {
            if (!_gameRunning()) return DianaShResult.Fail("No ROM loaded - this command needs an active debug target.");
            _openWindow();
            return DianaShResult.Ok("coretop: opened in a separate window.");
        }
    }

    // Same replacement as coretop above, and takes no target at all - see `man vstop`.
    public class VstopWindowCommand : HostCommand
    {
        private readonly Action _openWindow;

        public VstopWindowCommand(Action openWindow)
        {
            _openWindow = openWindow;
        }

        public override string Name => "vstop";
        public override bool IsReadOnly => true;
        public override string Usage => "  vstop                         open a live .NET runtime dashboard window (non-blocking - gameplay keeps running)";

        protected override DianaShResult Run(string[] args, string? stdin)
        {
            _openWindow();
            return DianaShResult.Ok("vstop: opened in a separate window.");
        }
    }

    // The picture is already on screen here, so this only raises the window - see `man feed`.
    public class FeedCommand : HostCommand
    {
        private readonly Action _bringGameWindowForward;

        public FeedCommand(Action bringGameWindowForward)
        {
            _bringGameWindowForward = bringGameWindowForward;
        }

        public override string Name => "feed";
        public override bool IsReadOnly => true;
        public override string Usage => "  feed [-w]                     bring the game window to the front (already showing the live feed - '-w' is accepted and ignored here)";

        protected override DianaShResult Run(string[] args, string? stdin)
        {
            _bringGameWindowForward();
            return DianaShResult.Ok("feed: game window brought to the front.");
        }
    }
}
