using System;
using System.IO;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.WiseMan.Fixtures
{
    // Shared DianaOS shell test setup - see EmuSen_Debugging_Tools_Reference_v5.md §3.18.
    public static class DianaShellFixtures
    {
        public static (DianaOSInterpreter Shell, DianaOSSessionManager Sessions) NewShellWithSessions(IDebugTarget? target = null)
        {
            var sessions = new DianaOSSessionManager();
            var shell = DianaOSInterpreter.CreateDefault(target, sessions: sessions);
            sessions.RegisterInitial(new DianaOSSession("main", shell));
            return (shell, sessions);
        }

        public sealed class ScratchDir : IDisposable
        {
            public string Path { get; }

            public ScratchDir(string subfolder)
            {
                Path = System.IO.Path.Combine(DianaOSSandbox.RootDirectory, "var", "log", subfolder, $"run_{Guid.NewGuid():N}");
                Directory.CreateDirectory(Path);
            }

            public void Dispose() => Directory.Delete(Path, recursive: true);
        }
    }
}
