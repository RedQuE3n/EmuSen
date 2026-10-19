using System;
using System.Diagnostics;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client.Contract;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Crystal
{
    // P2 at B7: a frame and its drain through each implementation's C ABI against the C# engine's RunFrame called directly, on the v1 test core; set EMUSEN_CRYSTAL_P2=1 and run in Release - see EmuSen_Backend.md §29.16.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed unsafe class CrystalFrameCostTool(ITestOutputHelper output) : IDisposable
    {
        private const int Frames = 20_000, Rounds = 5;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCrystalP2_" + Guid.NewGuid().ToString("N"));

        public void Dispose()
        {
            CoreDiscovery.UseDirectories(null);
            global::EmuSen.Galaxia.Library.DataStore.OverrideDirectory = null;
            global::EmuSen.Galaxia.ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        // The best of five rounds, in microseconds a frame.
        private static double Best(Action<int> frame)
        {
            for (int i = 0; i < 2_000; i++) frame(i);
            double best = double.MaxValue;
            for (int r = 0; r < Rounds; r++)
            {
                long start = Stopwatch.GetTimestamp();
                for (int i = 0; i < Frames; i++) frame(i);
                best = Math.Min(best, Stopwatch.GetElapsedTime(start).TotalMicroseconds / Frames);
            }
            return best;
        }

        [Fact]
        public void The_frame_call_costs_against_run_frame()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_CRYSTAL_P2") != "1") return;
            Directory.CreateDirectory(_root);
            global::EmuSen.Galaxia.Library.DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            global::EmuSen.Galaxia.ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            string cores = Path.Combine(_root, "cores");
            Directory.CreateDirectory(cores);
            string library = Path.Combine(cores, Path.GetFileName(CoreAdapterTests.LibraryPath("v1_test_core")));
            File.Copy(CoreAdapterTests.LibraryPath("v1_test_core"), library);
            CoreSidecar.Write(library, false);
            File.WriteAllText(Path.Combine(cores, "cores.manifest"), $"{CoreSidecar.HashOf(library)}  {Path.GetFileName(library)}\n");
            CoreDiscovery.UseDirectories(new[] { cores });
            string game = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(game, CoreAdapterTests.Image(1, 2, 3, 4));
            var sound = new short[8192];

            CoreBundle bundle = CoreFactory.Load(game, headless: true);
            var engine = (CoreEngine)bundle.Core;
            double direct = Best(_ =>
            {
                engine.RunFrame();
                bundle.DebugTarget.RefreshProviders();
                engine.DrainAudio(sound, out _);
            });
            (bundle.Core as IDisposable)?.Dispose();

            string Through(CrystalImpl impl, out double cost)
            {
                nint rt = impl.New(new[] { cores });
                uint s = uint.Parse(impl.Open(rt, game));
                var frame = (delegate* unmanaged[Cdecl]<nint, uint, CrystalFrameInfo*, int>)CrystalExportOf(impl, "frame");
                var drain = (delegate* unmanaged[Cdecl]<nint, uint, short*, nuint, uint*, long>)CrystalExportOf(impl, "audio_drain");
                var infos = new CrystalFrameInfo[1];
                infos[0].size = (uint)sizeof(CrystalFrameInfo);
                fixed (short* o = sound)
                fixed (CrystalFrameInfo* i = infos)
                {
                    short* at = o;
                    CrystalFrameInfo* info = i;
                    cost = Best(_ =>
                    {
                        if (frame(rt, s, info) != 0) throw new InvalidOperationException(impl.LastError());
                        uint rate;
                        drain(rt, s, at, 4096, &rate);
                    });
                }
                impl.Call("close", rt, s);
                impl.Free(rt);
                return impl.Name;
            }
            Through(CrystalImpl.Managed, out double managed);
            Through(CrystalImpl.Native, out double native);
            output.WriteLine($"RunFrame, the refresh and the drain on the C# engine: {direct:F3} us a frame");
            output.WriteLine($"the C# frame call and drain through its exports: {managed:F3} us ({managed - direct:+0.000;-0.000})");
            output.WriteLine($"the Rust frame call and drain through its exports: {native:F3} us ({native - direct:+0.000;-0.000})");
        }

        private static nint CrystalExportOf(CrystalImpl impl, string name) => impl.Export(name);
    }
}
