using System.Diagnostics;
using EmuSen.Common;
using EmuSen.Cores.Native;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // A v1 machine entered from a second thread while its emulation thread runs, as the window's pad timer and the console enter it; and when a change reaches a frame - see Diamond_Players.md §12.7.
    [Collection(TestCollections.ProcessGlobals)]
    public unsafe class CoreEngineThreadTests : IDisposable
    {
        private const string MarkVariable = "EMUSEN_TEST_THREAD_MARK", ModeVariable = "EMUSEN_TEST_THREAD_MODE";
        private const int Frames = 900;

        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenThreads_" + Guid.NewGuid().ToString("N"));

        public CoreEngineThreadTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // A panic across the boundary aborts the process, so each run is a child test host judged by its exit and its mark.
        [Theory]
        [InlineData("input")]
        [InlineData("debugger")]
        public void A_second_thread_entering_a_running_machine_does_not_end_the_process(string mode)
        {
            var (finished, exit, said, _) = Child(mode);
            Assert.True(finished, "the child test run did not finish");
            Assert.False(said.Contains("already borrowed", StringComparison.Ordinal), said);
            Assert.True(exit == 0, said);
            string mark = Path.Combine(_root, "mark.txt");
            Assert.True(File.Exists(mark), "the child ran nothing:\n" + said);
            string[] result = File.ReadAllLines(mark);
            _output.WriteLine(string.Join(" | ", result));
            Assert.Equal($"frames {Frames}", result[0]);
        }

        // An export entered from a second thread past the gate, as a host ignoring §6.16 would: the core's panic hook leaves its report in the log folder, which nothing else had made.
        [Fact]
        public void A_panic_in_the_core_leaves_its_crash_file_in_the_log_folder()
        {
            var (finished, exit, said, crashFiles) = Child("raw");
            Assert.True(finished, "the child test run did not finish");
            Assert.True(exit != 0 && said.Contains("already borrowed", StringComparison.Ordinal), said);
            string file = Assert.Single(crashFiles);
            Assert.Equal(Path.Combine(_root, "Logs"), Path.GetDirectoryName(file));
            Assert.StartsWith("diamond_crash_", Path.GetFileName(file));
            string report = File.ReadAllText(file);
            Assert.Contains("Diamond panic", report);
            Assert.Contains("RefCell already borrowed", report);
        }

        private (bool Finished, int Exit, string Said, string[] CrashFiles) Child(string mode)
        {
            string mark = Path.Combine(_root, "mark.txt");
            var start = new ProcessStartInfo("dotnet") { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false, WorkingDirectory = _root };
            foreach (string argument in new[] { "test", typeof(CoreEngineThreadTests).Assembly.Location, "--filter", $"FullyQualifiedName={typeof(CoreEngineThreadTests).FullName}.{nameof(TheMachineEnteredFromASecondThread)}" })
                start.ArgumentList.Add(argument);
            start.Environment["DOTNET_CLI_TELEMETRY_OPTOUT"] = "1";
            start.Environment["DOTNET_NOLOGO"] = "1";
            start.Environment["DOTNET_SKIP_FIRST_TIME_EXPERIENCE"] = "1";
            start.Environment[MarkVariable] = mark;
            start.Environment[ModeVariable] = mode;

            using Process child = Process.Start(start)!;
            Task<string> output = child.StandardOutput.ReadToEndAsync();
            Task<string> errors = child.StandardError.ReadToEndAsync();
            bool finished = child.WaitForExit(240_000);
            if (!finished) child.Kill(entireProcessTree: true);
            string said = output.Result + errors.Result;
            string[] crashFiles = Directory.GetFiles(_root, "*_crash_*", SearchOption.AllDirectories);
            int exit = finished ? child.ExitCode : -1;
            _output.WriteLine($"exit {exit}; crash files: {string.Join(", ", crashFiles.Select(Path.GetFileName))}");
            return (finished, exit, said, crashFiles);
        }

        // Run only by the test above, in its own process; it does nothing unless that process set the mark.
        [Fact]
        public void TheMachineEnteredFromASecondThread()
        {
            if (Environment.GetEnvironmentVariable(MarkVariable) is not { } mark || Environment.GetEnvironmentVariable(ModeVariable) is not { } mode) return;
            string root = Path.GetDirectoryName(mark)!;
            string? logsWere = ErrorLog.DirectoryOverride;
            DataStore.OverrideDirectory = Path.Combine(root, "Home");
            ErrorLog.DirectoryOverride = Path.Combine(root, "Logs");
            CoreDiscovery.UseDevelopment(true);
            try
            {
                string exe = Path.Combine(root, "pad.psexe");
                File.WriteAllBytes(exe, DiamondTests.AnalogPadReading());
                var session = new EmulatorSession();
                session.LoadRom(exe);
                Assert.IsType<CoreEngine>(session.Core);
                Assert.Contains(PadAxis.LeftX, session.SupportedAxes);
                var target = session.DebugTarget!;
                var shell = DianaShInterpreter.CreateDefault(target);
                var scheduler = new DianaShInterpreterScheduler();
                CoreMachine machine = ((CoreEngine)session.Core!).Machine;

                // The window's pad timer: every axis on each poll, a button on each change; or the console's read-only lines and the dashboard's reads.
                bool stop = false;
                long calls = 0;
                var other = new Thread(() =>
                {
                    for (long n = 0; !Volatile.Read(ref stop); n++)
                    {
                        if (mode == "input")
                        {
                            session.SetAxis(0, PadAxis.LeftX, (n & 1) == 0 ? 0.5 : -0.5);
                            session.SetAxis(0, PadAxis.LeftY, 0.25);
                            session.SetButton(0, PadButton.Start, (n & 2) == 0);
                        }
                        else if (mode == "raw")
                        {
                            machine.Library.Api.SetAxis(machine.Handle, 0, 0, (n & 1) == 0 ? 0.5 : -0.5);
                        }
                        else
                        {
                            scheduler.SubmitFromAnyThread(shell, "mem ram 10000 16");
                            scheduler.SubmitFromAnyThread(shell, "disasm r3000");
                            _ = target.FrameCount;
                            _ = target.GetSummaryText();
                        }
                        calls++;
                    }
                }) { IsBackground = true, Name = "second" };

                // The emulation thread's own order: the frame, then the debugger's snapshots.
                var clock = Stopwatch.StartNew();
                other.Start();
                int frames = 0;
                while (frames < Frames && clock.Elapsed < TimeSpan.FromSeconds(60))
                {
                    session.RunFrame();
                    target.RefreshProviders();
                    frames++;
                }
                Volatile.Write(ref stop, true);
                other.Join();
                File.WriteAllLines(mark, new[] { $"frames {frames}", $"calls {calls}", $"seconds {clock.Elapsed.TotalSeconds:F1}" });
                (session.Core as IDisposable)?.Dispose();
            }
            finally
            {
                CoreDiscovery.UseDevelopment(null);
                DataStore.OverrideDirectory = null;
                ErrorLog.DirectoryOverride = logsWere;
            }
        }

        // A change made before a frame starts, on another thread, is what that frame's pad reads show; and so is its undoing.
        [Fact]
        public void A_press_and_a_stick_set_before_a_frame_reach_that_frame()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            try
            {
                string exe = Path.Combine(_root, "pad.psexe");
                File.WriteAllBytes(exe, DiamondTests.AnalogPadReading());
                using var engine = new CoreEngine(CoreLibrary.Open(DiamondTests.LibraryPath));
                engine.LoadRom(exe);
                byte[] Reply()
                {
                    var reply = new byte[9];
                    for (int i = 0; i < reply.Length; i++) reply[i] = engine.ReadSpace("ram", 0x10_0000 + i);
                    return reply;
                }
                void Elsewhere(Action change)
                {
                    var t = new Thread(() => change());
                    t.Start();
                    t.Join();
                }

                for (int i = 0; i < 3; i++) engine.RunFrame();
                Assert.Equal(new byte[] { 0xFF, 0x73, 0x5A, 0xFF, 0xFF, 0x80, 0x80, 0x80, 0x80 }, Reply());

                Elsewhere(() =>
                {
                    engine.SetButton(0, PadButton.Start, true);
                    engine.SetAxis(0, PadAxis.LeftX, 1.0);
                    engine.SetAxis(0, PadAxis.RightY, -1.0);
                });
                engine.RunFrame();
                Assert.Equal(new byte[] { 0xFF, 0x73, 0x5A, 0xF7, 0xFF, 0x80, 0x00, 0xFF, 0x80 }, Reply());

                Elsewhere(() =>
                {
                    engine.SetButton(0, PadButton.Start, false);
                    engine.SetAxis(0, PadAxis.LeftX, 0);
                    engine.SetAxis(0, PadAxis.RightY, 0);
                });
                engine.RunFrame();
                Assert.Equal(new byte[] { 0xFF, 0x73, 0x5A, 0xFF, 0xFF, 0x80, 0x80, 0x80, 0x80 }, Reply());
            }
            finally
            {
                DataStore.OverrideDirectory = null;
            }
        }
    }
}
