using System.IO;
using System.Linq;
using EmuSen.Galaxia.Media;

namespace EmuSen.WiseMan.Galaxia
{
    // A CHD beside its original claims the original only while the tool's record verifies it, by both halves - see EmuSen_GamePackages.md §20.3.
    public partial class GalaxiaParityTests
    {
        private static string BesideManifest(string original, string chd, string tracks)
        {
            var (size, mtime) = GamePackages.Managed.SeenOf(original)!.Value;
            var (csize, cmtime) = GamePackages.Managed.SeenOf(chd)!.Value;
            return "{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"psx\", \"mode\": \"linked\", " +
                $"\"game\": {{\"role\": \"image\", \"where\": \"library\", \"path\": \"psx/Disc.chd\", \"size\": {csize}, \"seen\": {{\"size\": {csize}, \"mtime\": \"{cmtime}\"}}}}, " +
                $"\"container\": {{\"type\": \"chd\", \"sha1\": \"ab\", \"sources\": [{{\"where\": \"library\", \"path\": \"psx/Disc.bin\", \"size\": {size}, \"mtime\": \"{mtime}\"}}], " +
                $"\"checks\": {{\"chdman\": \"passed\", \"tracks\": \"{tracks}\", \"toc\": \"passed\"}}}}}}";
        }

        [Fact]
        public void A_chd_beside_its_original_claims_it_only_while_verified_both_ways()
        {
            string root = At("beside", "index"), library = Path.Combine(root, "Roms"), packages = Path.Combine(root, "Packages");
            var places = new PackagePlaces(packages, library, Path.Combine(root, "Compressed"));
            string original = Path.Combine(library, "psx", "Disc.bin"), chd = Path.Combine(library, "psx", "Disc.chd");
            File.WriteAllBytes(Put(original), Cartridge(5, 2352));
            File.WriteAllBytes(Put(chd), Cartridge(6, 100));
            string manifest = Put(Path.Combine(packages, "PlayStation", "Disc.chd", GamePackage.FileName));

            File.WriteAllText(manifest, BesideManifest(original, chd, "passed"));
            Assert.Equal(new[] { chd, original }, GamePackages.Managed.Index(places).Packages.Single().Claims);
            Assert.Equal(Shown(GamePackages.Managed.Index(places)), Shown(GamePackages.Index(places)));

            File.WriteAllText(manifest, BesideManifest(original, chd, "failed: track 1"));
            Assert.Equal(new[] { chd }, GamePackages.Managed.Index(places).Packages.Single().Claims);
            Assert.Equal(Shown(GamePackages.Managed.Index(places)), Shown(GamePackages.Index(places)));

            File.WriteAllText(manifest, BesideManifest(original, chd, "passed"));
            File.SetLastWriteTimeUtc(original, File.GetLastWriteTimeUtc(original).AddHours(2));
            Assert.Equal(new[] { chd }, GamePackages.Managed.Index(places).Packages.Single().Claims);
            Assert.Equal(Shown(GamePackages.Managed.Index(places)), Shown(GamePackages.Index(places)));
        }
    }
}
