using System;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.Mistress.Views
{
    // Mistress-only: acts on the emulation thread, not an IDebugTarget - see `man pause`.
    public class PauseCommand : IDianaShCommand
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

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (_isPaused()) return DianaShResult.Ok("Already paused.");
            _pause();
            return DianaShResult.Ok("Emulation paused.");
        }
    }

    public class ResumeCommand : IDianaShCommand
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

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (!_isPaused()) return DianaShResult.Ok("Not paused.");
            _resume();
            return DianaShResult.Ok("Emulation resumed.");
        }
    }

    // Replaces the raw-terminal coretop, which cannot work in a TextBox - see `man coretop`.
    public class CoretopWindowCommand : IDianaShCommand
    {
        private readonly Action<IDebugTarget?> _openWindow;

        public CoretopWindowCommand(Action<IDebugTarget?> openWindow)
        {
            _openWindow = openWindow;
        }

        public string Name => "coretop";
        public bool IsReadOnly => true;
        public string Usage => "  coretop                       open a live hardware dashboard window (non-blocking - gameplay keeps running)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (target is null) return DianaShResult.Fail("No ROM loaded - this command needs an active debug target.");
            _openWindow(target);
            return DianaShResult.Ok("coretop: opened in a separate window.");
        }
    }

    // Same replacement as coretop above, and takes no target at all - see `man vstop`.
    public class VstopWindowCommand : IDianaShCommand
    {
        private readonly Action _openWindow;

        public VstopWindowCommand(Action openWindow)
        {
            _openWindow = openWindow;
        }

        public string Name => "vstop";
        public bool IsReadOnly => true;
        public string Usage => "  vstop                         open a live .NET runtime dashboard window (non-blocking - gameplay keeps running)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _openWindow();
            return DianaShResult.Ok("vstop: opened in a separate window.");
        }
    }

    // The picture is already on screen here, so this only raises the window - see `man feed`.
    public class FeedCommand : IDianaShCommand
    {
        private readonly Action _bringGameWindowForward;

        public FeedCommand(Action bringGameWindowForward)
        {
            _bringGameWindowForward = bringGameWindowForward;
        }

        public string Name => "feed";
        public bool IsReadOnly => true;
        public string Usage => "  feed [-w]                     bring the game window to the front (already showing the live feed - '-w' is accepted and ignored here)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _bringGameWindowForward();
            return DianaShResult.Ok("feed: game window brought to the front.");
        }
    }
}
