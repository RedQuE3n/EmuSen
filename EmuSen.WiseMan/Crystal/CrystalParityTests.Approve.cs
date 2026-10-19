using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client.Contract;
using EmuSen.WiseMan.Cores;

namespace EmuSen.WiseMan.Crystal
{
    // The frontend API's 1.3: the player's directory, core_approve and core_revoke, their answers and the approvals' file compared - see EmuSen_Backend.md §29.40.
    public partial class CrystalParityTests
    {
        // Each implementation over installation and player's directories, the approvals' file and the asks' leftovers cleared first by the asks.
        private List<string> SameWithPlayer(IReadOnlyList<string> cores, IReadOnlyList<string> player, Func<CrystalImpl, nint, IEnumerable<string>> asks)
        {
            var runs = new List<List<string>>();
            foreach (CrystalImpl impl in new[] { CrystalImpl.Managed, CrystalImpl.Native })
            {
                CoreDiscovery.UseDevelopment(null);
                CoreDiscovery.UseDirectories(cores);
                CoreDiscovery.UsePlayerDirectories(player);
                nint rt = impl.New(cores, false, player);
                try { runs.Add(asks(impl, rt).ToList()); }
                finally
                {
                    impl.Free(rt);
                    CoreDiscovery.UsePlayerDirectories(null);
                }
            }
            Compare(runs[0], runs[1]);
            return runs[0];
        }

        // A test core copied into dir with its sidecar written in a build folder, so the copy is never loaded by the writing.
        private string Installed(string name, string dir)
        {
            string build = Path.Combine(_root, "build" + Guid.NewGuid().ToString("N")[..6]);
            Directory.CreateDirectory(build);
            Directory.CreateDirectory(dir);
            string built = Path.Combine(build, Path.GetFileName(CoreAdapterTests.LibraryPath(name)));
            File.Copy(CoreAdapterTests.LibraryPath(name), built);
            CoreSidecar.Write(built);
            string copy = Path.Combine(dir, Path.GetFileName(built));
            File.Copy(built, copy);
            File.Copy(CoreSidecar.PathFor(built), CoreSidecar.PathFor(copy));
            return copy;
        }

        private static string ApprovalsText() => File.Exists(CoreTrust.ApprovalsPath) ? File.ReadAllText(CoreTrust.ApprovalsPath) : "(none)";

        [Fact]
        public void Approving_and_revoking_a_players_core_answer_and_write_the_same()
        {
            string tree = TestTree(plain: false);
            string player = Path.Combine(_root, "Cores");
            string plain = Installed("v1_plain_core", player);
            string hash = CoreSidecar.HashOf(plain);
            string game = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(game, CoreAdapterTests.Image(1, 2, 3, 4));
            const string engine = "V1PlainCore (test)";
            List<string> a0 = SameWithPlayer(new[] { tree }, new[] { player }, (impl, rt) =>
            {
                if (File.Exists(CoreTrust.ApprovalsPath)) File.Delete(CoreTrust.ApprovalsPath);
                impl.Rescan(rt);
                var a = new List<string> { impl.Read("cores", rt), impl.Read("engines", rt) };
                a.Add(impl.Open(rt, game, engine: engine, flags: CrystalAbi.OpenNoBattery));
                a.Add(impl.TrustChange("core_approve", rt, "not a hash"));
                a.Add(impl.TrustChange("core_approve", rt, new string('0', 64)));
                a.Add(impl.TrustChange("core_revoke", rt, hash));
                a.Add(ApprovalsText());
                a.Add(impl.TrustChange("core_approve", rt, hash.ToUpperInvariant()));
                a.Add(ApprovalsText());
                a.Add(impl.TrustChange("core_approve", rt, hash));
                a.Add(ApprovalsText());
                a.Add(impl.Read("cores", rt));
                a.Add(impl.Read("engines", rt));
                string opened = impl.Open(rt, game, engine: engine, flags: CrystalAbi.OpenNoBattery);
                a.Add(opened.StartsWith("status", StringComparison.Ordinal) ? opened : "opened");
                if (!opened.StartsWith("status", StringComparison.Ordinal)) a.Add(impl.Call("close", rt, uint.Parse(opened)));
                a.Add(impl.TrustChange("core_revoke", rt, hash));
                a.Add(ApprovalsText());
                a.Add(impl.Read("cores", rt));
                return a;
            });
            Assert.Equal("", File.ReadAllText(CoreTrust.ApprovalsPath));
            string line = $"{hash}  {Path.GetFileName(plain)}\n";
            Assert.Equal(new[] { $"status {CrystalAbi.NotApproved}: {Path.GetFileName(plain)} is in no cores manifest and has not been approved; it was not loaded" }, a0.Skip(2).Take(1));
            Assert.StartsWith($"status {CrystalSessionAbi.BadArgument}:", a0[3]);
            Assert.StartsWith($"status {CrystalSessionAbi.NoSuchId}:", a0[4]);
            Assert.StartsWith($"status {CrystalSessionAbi.NoSuchId}:", a0[5]);
            Assert.Equal(new[] { "(none)", "0", line, "0", line }, a0.Skip(6).Take(5));
            Assert.Contains("\"trust\":\"approved\"", a0[11]);
            Assert.Equal(new[] { "opened", "0", "0", "" }, a0.Skip(13).Take(4));
        }

        [Fact]
        public void A_players_library_without_a_sidecar_is_read_on_approval_the_same_by_both()
        {
            string player = Path.Combine(_root, "Bare");
            Directory.CreateDirectory(player);
            string bare = Path.Combine(player, Path.GetFileName(CoreAdapterTests.LibraryPath("v1_plain_core")));
            File.Copy(CoreAdapterTests.LibraryPath("v1_plain_core"), bare);
            string hash = CoreSidecar.HashOf(bare);
            List<string> b0 = SameWithPlayer(Array.Empty<string>(), new[] { player }, (impl, rt) =>
            {
                if (File.Exists(CoreTrust.ApprovalsPath)) File.Delete(CoreTrust.ApprovalsPath);
                File.Delete(CoreSidecar.PathFor(bare));
                impl.Rescan(rt);
                var a = new List<string> { impl.Read("cores", rt), impl.Read("engines", rt) };
                a.Add(impl.TrustChange("core_approve", rt, hash));
                a.Add(File.Exists(CoreSidecar.PathFor(bare)) ? "sidecar written" : "no sidecar");
                a.Add(ApprovalsText());
                a.Add(impl.Read("cores", rt));
                a.Add(impl.Read("engines", rt));
                return a;
            });
            Assert.True(File.Exists(CoreSidecar.PathFor(bare)));
            Assert.Contains("\"trust\":\"not_read\"", b0[0]);
            Assert.Equal("[]", b0[1]);
            Assert.Equal(new[] { "0", "sidecar written" }, b0.Skip(2).Take(2));
            Assert.Contains("\"trust\":\"approved\"", b0[6]);
        }
    }
}
