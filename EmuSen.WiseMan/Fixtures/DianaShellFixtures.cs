using System;
using System.IO;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.WiseMan.Fixtures
{
    // Shared DianaSh shell test setup - see EmuSen_Debugging_Tools_Reference_v5.md §3.18.
    public static class DianaShellFixtures
    {
        public static (DianaShInterpreter Shell, DianaShSessionManager Sessions) NewShellWithSessions(IDebugTarget? target = null)
        {
            var sessions = new DianaShSessionManager();
            var shell = DianaShInterpreter.CreateDefault(target, sessions: sessions);
            sessions.RegisterInitial(new DianaShSession("main", shell));
            return (shell, sessions);
        }

        public sealed class ScratchDir : IDisposable
        {
            public string Path { get; }

            public ScratchDir(string subfolder)
            {
                Path = System.IO.Path.Combine(DianaShSandbox.ScratchDirectory, subfolder, $"run_{Guid.NewGuid():N}");
                Directory.CreateDirectory(Path);
            }

            public void Dispose() => Directory.Delete(Path, recursive: true);
        }
    }
}
