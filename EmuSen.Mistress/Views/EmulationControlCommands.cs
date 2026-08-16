using System;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Mistress.Views
{
    // Mistress-only: acts on the emulation thread, not an IDebugTarget - see `man pause`.
    public class PauseCommand : IDianaOSCommand
    {
        private readonly Action _pause;
        private readonly Func<bool> _isPaused;

        public PauseCommand(Action pause, Func<bool> isPaused)
        {
            _pause = pause;
            _isPaused = isPaused;
        }

        public string Name => "pause";
        public bool IsReadOnly => false;
        public string Usage => "  pause                         pause the emulation thread (safe to read/write emulator state while paused)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (_isPaused()) return DianaOSResult.Ok("Already paused.");
            _pause();
            return DianaOSResult.Ok("Emulation paused.");
        }
    }

    public class ResumeCommand : IDianaOSCommand
    {
        private readonly Action _resume;
        private readonly Func<bool> _isPaused;

        public ResumeCommand(Action resume, Func<bool> isPaused)
        {
            _resume = resume;
            _isPaused = isPaused;
        }

        public string Name => "resume";
        public bool IsReadOnly => false;
        public string Usage => "  resume                        resume the emulation thread after a pause";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (!_isPaused()) return DianaOSResult.Ok("Not paused.");
            _resume();
            return DianaOSResult.Ok("Emulation resumed.");
        }
    }

    // Replaces the raw-terminal coretop, which cannot work in a TextBox - see `man coretop`.
    public class CoretopWindowCommand : IDianaOSCommand
    {
        private readonly Action<IDebugTarget?> _openWindow;

        public CoretopWindowCommand(Action<IDebugTarget?> openWindow)
        {
            _openWindow = openWindow;
        }

        public string Name => "coretop";
        public bool IsReadOnly => true;
        public string Usage => "  coretop                       open a live hardware dashboard window (non-blocking - gameplay keeps running)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (target is null) return DianaOSResult.Fail("No ROM loaded - this command needs an active debug target.");
            _openWindow(target);
            return DianaOSResult.Ok("coretop: opened in a separate window.");
        }
    }

    // Same replacement as coretop above, and takes no target at all - see `man vstop`.
    public class VstopWindowCommand : IDianaOSCommand
    {
        private readonly Action _openWindow;

        public VstopWindowCommand(Action openWindow)
        {
            _openWindow = openWindow;
        }

        public string Name => "vstop";
        public bool IsReadOnly => true;
        public string Usage => "  vstop                         open a live .NET runtime dashboard window (non-blocking - gameplay keeps running)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _openWindow();
            return DianaOSResult.Ok("vstop: opened in a separate window.");
        }
    }

    // The picture is already on screen here, so this only raises the window - see `man feed`.
    public class FeedCommand : IDianaOSCommand
    {
        private readonly Action _bringGameWindowForward;

        public FeedCommand(Action bringGameWindowForward)
        {
            _bringGameWindowForward = bringGameWindowForward;
        }

        public string Name => "feed";
        public bool IsReadOnly => true;
        public string Usage => "  feed [-w]                     bring the game window to the front (already showing the live feed - '-w' is accepted and ignored here)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _bringGameWindowForward();
            return DianaOSResult.Ok("feed: game window brought to the front.");
        }
    }
}
