using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Crystal.Debugger;

namespace EmuSen.WiseMan.Fixtures
{
    // The targets the golden corpus and the session service's parity run attach to, each built as a frontend builds it - see EmuSen_DianaSh.md §13.4 and §13.6.
    public sealed class GoldenTargets(string directory)
    {
        // Every core the corpus names, on the images the tests build.
        public static readonly IReadOnlyList<string> Cores = new[] { "venusrt", "venusrt-sa1", "venusrt-gsu", "moonrt", "mercuryrt", "marsrt", "nephrite" };

        private readonly Dictionary<string, string> _images = new();

        public sealed class Built(IDebugTarget? target, CoreBundle? bundle, IDisposable? owner) : IDisposable
        {
            public IDebugTarget? Target { get; } = target;
            public CoreBundle? Bundle { get; } = bundle;
            public void Dispose() => owner?.Dispose();
        }

        public Built Build(string name, int frames)
        {
            switch (name)
            {
                case "none": return new Built(null, null, null);
                case "fake": return new Built(new GoldenDebugTarget(), null, null);
                case "bare": return new Built(new GoldenDebugTarget(bare: true), null, null);
                case "nobus": return new Built(new GoldenDebugTarget(bus: false), null, null);
            }
            string path = Image(name);
            ICore core = CoreFactory.Create(path, headless: true, EngineName(name));
            core.LoadRom(path);
            for (int i = 0; i < frames; i++) core.RunFrame();
            CoreBundle bundle = CoreFactory.Bundle(core);
            return new Built(bundle.DebugTarget, bundle, core as IDisposable);
        }

        private string Image(string engine)
        {
            if (_images.TryGetValue(engine, out string? path)) return path;
            (string extension, byte[] bytes) = engine switch
            {
                "venusrt" => (".sfc", SyntheticRom.BuildBlank()),
                "venusrt-sa1" => (".sfc", SyntheticRom.BuildSa1()),
                "venusrt-gsu" => (".sfc", SyntheticRom.BuildSuperFx()),
                "moonrt" => (".nes", SyntheticNesRom.Build()),
                "mercuryrt" => (".gb", SyntheticGbRom.Build()),
                "marsrt" => (".z64", SyntheticN64Rom.Build(patches: (0, new byte[] { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 }))),
                "nephrite" => (".md", SyntheticMdRom.Cartridge()),
                _ => throw new ArgumentException($"no golden target '{engine}'"),
            };
            Directory.CreateDirectory(Path.Combine(directory, "images"));
            path = Path.Combine(directory, "images", engine + extension);
            File.WriteAllBytes(path, bytes);
            return _images[engine] = path;
        }

        private static string? EngineName(string engine) => engine switch
        {
            "venusrt" or "venusrt-sa1" or "venusrt-gsu" => CoreCatalog.VenusRtEngine,
            "moonrt" => CoreCatalog.MoonRtEngine,
            "mercuryrt" => CoreCatalog.MercuryRtEngine,
            "marsrt" => CoreCatalog.MarsRtEngine,
            _ => null,
        };
    }
}
