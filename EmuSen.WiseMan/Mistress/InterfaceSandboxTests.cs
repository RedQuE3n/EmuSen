using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A recording names a run's sandbox the same whichever numbered slot the run claimed, so two runs side by side record alike - see EmuSen_InterfaceSpec.md §6.1.
    [Collection(TestCollections.ProcessGlobals)]
    public class InterfaceSandboxTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(InterfaceSandboxTests).GetTypeInfo().Assembly);

        // A state whose tile is named by its game's path under the sandbox at root, as the hub's tiles are, recorded with normalise.
        private static string Recorded(string root, Func<string, string> normalise)
        {
            Func<string, string> was = InterfaceRecorder.Normalise;
            var tile = new Button { Name = $"HubTile.{root}/Roms/Aurora Drift (Synthetic).sfc", Content = "Aurora Drift (Synthetic)" };
            var window = new Window { Width = 640, Height = 400, Content = new StackPanel { Children = { tile } } };
            try
            {
                InterfaceRecorder.Normalise = normalise;
                window.Show();
                Dispatcher.UIThread.RunJobs();
                return InterfaceRecorder.Tree(window, new InterfaceRecorder.Options("Sandbox", "window")).Write();
            }
            finally
            {
                InterfaceRecorder.Normalise = was;
                window.Close();
            }
        }

        [Fact]
        public Task A_state_recorded_in_sandbox_001_and_in_sandbox_007_records_alike() => Session.Dispatch(() =>
        {
            foreach (string parent in new[] { "/tmp", Path.GetTempPath().TrimEnd('/') })
                foreach (string name in new[] { "EmuSenWindowFit", "EmuSenThemedSession" })
                {
                    string first = Path.Combine(parent, name, "001"), seventh = Path.Combine(parent, name, "007");
                    string a = Recorded(first, InterfaceSpecFiles.Normaliser()), b = Recorded(seventh, InterfaceSpecFiles.Normaliser());
                    Assert.Equal(a, b);
                    Assert.Contains("HubTile.<sandbox>/Roms/", a);
                    Assert.DoesNotContain(name + "/0", a);
                    Assert.Equal(Recorded(first, InterfaceSpecFiles.Normaliser(first)), Recorded(seventh, InterfaceSpecFiles.Normaliser(seventh)));
                }
        }, default);
    }
}
