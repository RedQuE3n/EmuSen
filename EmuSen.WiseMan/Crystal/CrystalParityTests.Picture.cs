using System.Collections.Generic;
using System.IO;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // Piece 7 of the parity class: the picture before its palette, chosen and copied, and a session given no device lending nothing - see EmuSen_Backend.md §29.20.
    public partial class CrystalParityTests
    {
        [Fact]
        public void The_raw_pictures_are_the_same()
        {
            string tree = TestTree(plain: false);
            string game = Path.Combine(_root, "plain.tst");
            File.WriteAllBytes(game, CoreAdapterTests.Image(1, 2, 3, 4));
            string nes = Path.Combine(_root, "Plain.nes");
            File.WriteAllBytes(nes, SyntheticNesRom.Build());
            IEnumerable<string> Run(CrystalImpl i, nint rt, string path, string encoding)
            {
                var a = new List<string>();
                uint s = Id(i.Open(rt, path));
                a.AddRange(new[] { i.RawCopy(rt, s), i.PictureLend(rt, s).ToString(), i.RawSelect(rt, s, encoding), i.RawCopy(rt, s) });
                for (int f = 0; f < 5; f++) a.AddRange(new[] { i.Frame(rt, s), i.RawCopy(rt, s) });
                a.AddRange(new[] { i.RawCopy(rt, s, size: 20), i.RawCopy(rt, s, len: 3), i.RawSelect(rt, s, "no.such.encoding"), i.Frame(rt, s), i.RawCopy(rt, s) });
                a.AddRange(new[] { i.RawSelect(rt, s, encoding), i.Frame(rt, s), i.RawCopy(rt, s), i.RawSelect(rt, s, null), i.Frame(rt, s), i.RawCopy(rt, s) });
                a.AddRange(new[] { i.RawSelect(rt, s, encoding), i.Call("reset", rt, s), i.Frame(rt, s), i.RawCopy(rt, s), i.PictureLend(rt, s).ToString(), i.Call("close", rt, s), i.RawCopy(rt, s) });
                return a;
            }
            SameWithFiles(new[] { tree }, () => { }, (i, rt) => Run(i, rt, game, "x-test.red"));
            SameWithFiles(BuildTree, () => { }, (i, rt) => Run(i, rt, nes, "nes.ppu"));
        }
    }
}
