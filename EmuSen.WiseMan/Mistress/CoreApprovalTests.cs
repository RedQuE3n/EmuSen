using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Mistress asks the player once before a game opens on a core they added, and records the approval through the runtime; a shipped core is never asked about - see EmuSen_Backend.md §29.40.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreApprovalTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CoreApprovalTests).GetTypeInfo().Assembly);

        private static readonly FieldInfo Confirm = typeof(MainWindow).GetField("ConfirmCoreApproval", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realConfirm = Confirm.GetValue(null)!;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoreApproval_" + Guid.NewGuid().ToString("N"));
        private int _asked;
        private string? _message;

        public CoreApprovalTests()
        {
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            Confirm.SetValue(null, _realConfirm);
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UsePlayerDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private void Answer(bool approve) => Confirm.SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, _, message) => { _asked++; _message = message; return Task.FromResult(approve); }));

        private static Task Ask(MainWindow window, string rom) =>
            (Task)typeof(MainWindow).GetMethod("AskCoreApprovalAsync", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { rom })!;

        // The plain test core in a directory, its sidecar written in a build folder so the copy is never loaded by the writing.
        private string Core(string dir)
        {
            string build = Path.Combine(_root, "build" + Guid.NewGuid().ToString("N")[..6]);
            Directory.CreateDirectory(build);
            Directory.CreateDirectory(dir);
            string built = Path.Combine(build, Path.GetFileName(CoreAdapterTests.LibraryPath("v1_plain_core")));
            File.Copy(CoreAdapterTests.LibraryPath("v1_plain_core"), built);
            CoreSidecar.Write(built);
            string copy = Path.Combine(dir, Path.GetFileName(built));
            File.Copy(built, copy);
            File.Copy(CoreSidecar.PathFor(built), CoreSidecar.PathFor(copy));
            return copy;
        }

        private string Game()
        {
            string path = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(path, CoreAdapterTests.Image(1, 2, 3, 4));
            return path;
        }

    }
}
