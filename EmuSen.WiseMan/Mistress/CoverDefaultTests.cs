using System.Globalization;
using System.IO;
using System.Text.Json;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The C# front end's one copy of the default cover shape, held to the systems data that owns it - see EmuSen_RustPlatform.md §28.12.
    public class CoverDefaultTests
    {
        [Fact]
        public void The_default_cover_shape_is_the_systems_datas()
        {
            using JsonDocument table = JsonDocument.Parse(File.ReadAllText(Path.Combine(ShaderCompiler.RepositoryRoot, "Platform", "systems", "data", "table.json")));
            Assert.Equal(table.RootElement.GetProperty("cover_aspect").GetDouble(), RuntimeCatalog.DefaultCoverAspect);
        }

        [Fact]
        public void A_system_only_an_engine_names_gets_the_default_from_the_runtime()
        {
            Assert.All(RuntimeCatalog.Systems, s => Assert.True(s.CoverAspect is > 0, string.Create(CultureInfo.InvariantCulture, $"{s.Id} has no cover shape")));
        }
    }
}
