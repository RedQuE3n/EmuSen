using System;
using System.IO;
using EmuSen.Crystal.Client;
using EmuSen.Mistress;

namespace EmuSen.WiseMan.Fixtures
{
    // A synthetic game of a console open on a runtime of its own, with a cheat list attached as Mistress attaches one while the game runs - see EmuSen_Backend.md §14.13.
    public sealed class RuntimeGame : IDisposable
    {
        private readonly string _path;

        public RuntimeGame(string console)
        {
            _path = console switch
            {
                "SNES" => SyntheticRom.WriteTempFile(SyntheticRom.Build()),
                "NES" => SyntheticNesRom.WriteTemp(SyntheticNesRom.Build()),
                "N64" => SyntheticN64Rom.WriteTemp(SyntheticN64System.Build(rsp: false)),
                _ => throw new ArgumentException($"no synthetic game of the {console}", nameof(console)),
            };
            Runtime = new CrystalRuntime();
            Session = Runtime.Open(new CrystalOpen { Path = _path, NoBattery = true });
            Cheats.Attach(Session);
        }

        public CrystalRuntime Runtime { get; }
        public CrystalSession Session { get; }
        public CheatList Cheats { get; } = new();

        public void Dispose()
        {
            Cheats.Detach();
            Session.Dispose();
            Runtime.Dispose();
            try { File.Delete(_path); } catch (IOException) { }
        }
    }
}
