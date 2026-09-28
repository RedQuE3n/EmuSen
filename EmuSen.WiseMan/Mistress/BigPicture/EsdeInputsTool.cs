using System;
using System.IO;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Writes the inputs ES-DE and Mistress share, outside the repository: EMUSEN_ESDE_INPUTS=<scratch home> runs it - see EmuSen_BigPicture.md §13.8.
    public class EsdeInputsTool
    {
        [Fact]
        public System.Threading.Tasks.Task Write_the_dummy_roms_gamelists_and_media() => UiTest.Run(() =>
        {
            string? home = Environment.GetEnvironmentVariable("EMUSEN_ESDE_INPUTS");
            if (string.IsNullOrEmpty(home)) return;
            SyntheticLibrary.WriteMedia(SceneRenderTool.MediaRoot);
            SyntheticLibrary.WriteEsdeLibrary(Path.Combine(SceneRenderTool.Cache, "roms"), Path.Combine(home, "ES-DE", "gamelists"));
        });
    }
}
