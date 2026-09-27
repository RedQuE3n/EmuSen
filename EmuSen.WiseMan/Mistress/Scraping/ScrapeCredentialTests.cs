using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // §5.7: one redactor for every credential that could reach a log, the status line, CrashLog or an exception; the files where they live - see EmuSen_BigPicture.md §17.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeCredentialTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeCredentialTests", Guid.NewGuid().ToString("N"));

        private static readonly System.Reflection.FieldInfo EmbeddedSource =
            typeof(DeveloperCredentials).GetField("EmbeddedSource", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)!;

        private readonly object _realEmbedded = EmbeddedSource.GetValue(null)!;

        public void Dispose()
        {
            EmbeddedSource.SetValue(null, _realEmbedded);
            ConfigStore.OverrideDirectory = null;
            ConfigStore.OverrideLegacyDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        [Theory]
        [InlineData("https://api.screenscraper.fr/api2/jeuInfos.php?devid=abcd&devpassword=FAKEp%40ss&softname=EmuSen&ssid=member&sspassword=hunter22&md5=00",
                    "https://api.screenscraper.fr/api2/jeuInfos.php?devid=***&devpassword=***&softname=EmuSen&ssid=***&sspassword=***&md5=00")]
        [InlineData("{\"url\":\"https://neoclone.screenscraper.fr/api2/mediaJeu.php?DEVID=abcd&DevPassword=FAKEzz9\",\"x\":1}",
                    "{\"url\":\"https://neoclone.screenscraper.fr/api2/mediaJeu.php?DEVID=***&DevPassword=***\",\"x\":1}")]
        [InlineData("GET mediaJeu.php?ssid=bob sspassword=secret1 then", "GET mediaJeu.php?ssid=*** sspassword=*** then")]
        [InlineData("nothing to hide: md5=00 romnom=a.sfc", "nothing to hide: md5=00 romnom=a.sfc")]
        public void Every_credential_parameter_is_blanked_and_nothing_else(string text, string safe)
        {
            Assert.Equal(safe, ScrapeRedactor.Redact(text));
        }

        [Fact]
        public void A_credential_s_own_value_is_blanked_wherever_it_turns_up()
        {
            _ = new DeveloperCredentials("FAKEDEVLITERAL", "FAKEPASSLITERAL", "EmuSen-Test");
            Assert.Equal("the password *** was in a stack trace", ScrapeRedactor.Redact("the password FAKEPASSLITERAL was in a stack trace"));
        }

        [Fact]
        public void Neither_credential_object_names_its_secrets_when_printed()
        {
            string printed = $"{FakeScreenScraper.Developer} {new MemberAccount("FAKEMEMBERNAME", "FAKEMEMBERPASS")}";
            Assert.DoesNotContain(FakeScreenScraper.DevId, printed);
            Assert.DoesNotContain(FakeScreenScraper.DevPassword, printed);
            Assert.DoesNotContain("FAKEMEMBERNAME", printed);
            Assert.DoesNotContain("FAKEMEMBERPASS", printed);
        }

        [Fact]
        public async Task A_failed_request_s_detail_carries_no_credential()
        {
            var failing = new ScreenScraperClientTests.Answering(r => throw new HttpRequestException($"could not reach {r.RequestUri}"));
            var client = new ScreenScraperClient(new HttpClient(failing), new DeveloperCredentials("FAKEID9", "FAKEPW9", "EmuSen-Test"), new MemberAccount("FAKEM9", "FAKEMPW9"));
            JeuInfosAnswer answer = await client.JeuInfosAsync(4, new RomHashes("00", "00", "00", 1), "a.sfc", CancellationToken.None);
            Assert.Equal(ScrapeStatus.Failed, answer.Status);
            Assert.Contains("devpassword=***", answer.Detail);
            foreach (string secret in (string[])["FAKEID9", "FAKEPW9", "FAKEM9", "FAKEMPW9"]) Assert.DoesNotContain(secret, answer.Detail);

            var page = new ScreenScraperClientTests.Answering(r => new HttpResponseMessage(HttpStatusCode.Forbidden) { Content = new StringContent($"Refused: {r.RequestUri}") });
            JeuInfosAnswer refused = await new ScreenScraperClient(new HttpClient(page), new DeveloperCredentials("FAKEID9", "FAKEPW9", "EmuSen-Test"), null)
                .JeuInfosAsync(4, new RomHashes("00", "00", "00", 1), "a.sfc", CancellationToken.None);
            Assert.DoesNotContain("FAKEPW9", refused.Detail);
        }

        [Fact]
        public void The_developer_file_is_read_from_the_config_directory_first_then_from_the_legacy_one()
        {
            string config = Path.Combine(_root, "config"), legacy = Path.Combine(_root, "legacy");
            ConfigStore.OverrideDirectory = config;
            ConfigStore.OverrideLegacyDirectory = legacy;
            Assert.Null(DeveloperCredentials.Load());

            WriteDeveloper(legacy, "FAKELEGACY");
            Assert.Equal("FAKELEGACY", DeveloperCredentials.Load()!.DevId);
            WriteDeveloper(config, "FAKECONFIG");
            Assert.Equal("FAKECONFIG", DeveloperCredentials.Load()!.DevId);
            Assert.Equal([Path.Combine(config, DeveloperCredentials.FileName), Path.Combine(legacy, DeveloperCredentials.FileName)], DeveloperCredentials.Locations());
        }

        [Fact]
        public void A_test_that_moved_the_config_directory_never_reaches_the_real_legacy_one_and_a_half_file_is_none()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "config");
            Assert.Equal([Path.Combine(_root, "config", DeveloperCredentials.FileName)], DeveloperCredentials.Locations());

            Directory.CreateDirectory(Path.Combine(_root, "config"));
            File.WriteAllText(Path.Combine(_root, "config", DeveloperCredentials.FileName), "{\"devid\":\"FAKEHALF\",\"softname\":\"x\"}");
            Assert.Null(DeveloperCredentials.Load());
        }

        // --- Q40: the credentials a published build carries (§4.65) ---

        // The layout ScreenScraperDeveloper.targets writes; ScreenScraperEmbedTests checks the target's own output against the same decoder.
        internal static byte[] Scramble(string json, int keyLength = 32)
        {
            byte[] body = Encoding.UTF8.GetBytes(json), key = System.Security.Cryptography.RandomNumberGenerator.GetBytes(keyLength);
            byte[] blob = new byte[6 + key.Length + body.Length];
            "ESSD"u8.CopyTo(blob);
            blob[4] = 1;
            blob[5] = (byte)key.Length;
            key.CopyTo(blob, 6);
            for (int i = 0; i < body.Length; i++) blob[6 + key.Length + i] = (byte)(body[i] ^ key[i % key.Length]);
            return blob;
        }

        private void Embed(byte[]? blob) => EmbeddedSource.SetValue(null, (Func<Stream?>)(() => blob is null ? null : new MemoryStream(blob)));

        private static string DeveloperJson(string id, string password) => JsonSerializer.Serialize(new { devid = id, devpassword = password, softname = "EmuSen-Test" });

        [Fact]
        public void The_tree_s_file_comes_first_then_the_user_s_then_what_the_build_carries()
        {
            string config = Path.Combine(_root, "config"), legacy = Path.Combine(_root, "legacy");
            ConfigStore.OverrideDirectory = config;
            ConfigStore.OverrideLegacyDirectory = legacy;
            Embed(null);
            Assert.Null(DeveloperCredentials.Load());

            Embed(Scramble(DeveloperJson("FAKEEMBEDID", "FAKEEMBEDPW")));
            DeveloperCredentials embedded = DeveloperCredentials.Load()!;
            Assert.Equal(("FAKEEMBEDID", "FAKEEMBEDPW", "EmuSen-Test", DeveloperOrigin.Embedded), (embedded.DevId, embedded.DevPassword, embedded.SoftName, embedded.Origin));

            WriteDeveloper(legacy, "FAKELEGACY");
            Assert.Equal(("FAKELEGACY", DeveloperOrigin.UserFile), (DeveloperCredentials.Load()!.DevId, DeveloperCredentials.Load()!.Origin));
            WriteDeveloper(config, "FAKECONFIG");
            Assert.Equal(("FAKECONFIG", DeveloperOrigin.TreeFile), (DeveloperCredentials.Load()!.DevId, DeveloperCredentials.Load()!.Origin));

            // A broken file in the tree is passed over, not taken as the end of the search.
            File.WriteAllText(Path.Combine(config, DeveloperCredentials.FileName), "{\"devid\":");
            Assert.Equal("FAKELEGACY", DeveloperCredentials.Load()!.DevId);
            File.Delete(Path.Combine(legacy, DeveloperCredentials.FileName));
            Assert.Equal(DeveloperOrigin.Embedded, DeveloperCredentials.Load()!.Origin);
        }

        [Fact]
        public void The_embedded_credentials_are_redacted_everywhere_never_printed_and_never_written()
        {
            string config = Path.Combine(_root, "config"), legacy = Path.Combine(_root, "legacy");
            ConfigStore.OverrideDirectory = config;
            ConfigStore.OverrideLegacyDirectory = legacy;
            string id = "FAKEEMB" + Guid.NewGuid().ToString("N")[..8], password = "FAKEEMBPW&" + Guid.NewGuid().ToString("N")[..8];
            Embed(Scramble(DeveloperJson(id, password)));

            DeveloperCredentials developer = DeveloperCredentials.Load()!;
            Assert.Equal(DeveloperOrigin.Embedded, developer.Origin);
            Assert.Equal("a *** and *** and ***", ScrapeRedactor.Redact($"a {id} and {password} and {Uri.EscapeDataString(password)}"));
            Assert.DoesNotContain(password, developer.ToString());
            Assert.DoesNotContain(id, developer.ToString());
            Assert.False(Directory.Exists(config) && Directory.EnumerateFileSystemEntries(config).Any());
            Assert.False(Directory.Exists(legacy));

            // Nothing a player saves carries them either.
            new MemberAccount("FAKEMEMBERX", "FAKEMEMBERXPW").Save();
            new EmuSen.Galaxia.Models.AppSettings().Save();
            foreach (string file in Directory.EnumerateFiles(config, "*", SearchOption.AllDirectories))
                Assert.DoesNotContain(password, File.ReadAllText(file));
            Assert.False(File.Exists(Path.Combine(config, DeveloperCredentials.FileName)));
        }

        [Theory]
        [InlineData("wrong magic")]
        [InlineData("truncated")]
        [InlineData("no key")]
        [InlineData("not json")]
        [InlineData("half")]
        [InlineData("newer version")]
        public void A_blob_that_is_not_what_the_target_writes_is_no_credentials(string kind)
        {
            byte[] blob = kind switch
            {
                "not json" => Scramble("devid=FAKEX devpassword=FAKEY"),
                "half" => Scramble("{\"devid\":\"FAKEX\",\"softname\":\"x\"}"),
                _ => Scramble(DeveloperJson("FAKEX", "FAKEY")),
            };
            if (kind == "wrong magic") blob[0] = (byte)'X';
            if (kind == "truncated") blob = blob[..20];
            if (kind == "no key") blob[5] = 0;
            if (kind == "newer version") blob[4] = 2;
            Assert.Null(DeveloperCredentials.Decode(blob));
            Assert.NotNull(DeveloperCredentials.Decode(Scramble(DeveloperJson("FAKEX", "FAKEY"))));
        }

        [Fact]
        public void The_scrambled_bytes_hold_no_credential_in_plain_text()
        {
            byte[] blob = Scramble(DeveloperJson("FAKEPLAINID", "FAKEPLAINPASSWORD"));
            string asText = Encoding.UTF8.GetString(blob);
            Assert.DoesNotContain("FAKEPLAINPASSWORD", asText);
            Assert.DoesNotContain("FAKEPLAINID", asText);
            Assert.DoesNotContain("devpassword", asText);
            Assert.Equal("FAKEPLAINPASSWORD", DeveloperCredentials.Decode(blob)!.DevPassword);
        }

        [Fact]
        public void The_member_account_is_its_own_file_readable_only_by_its_owner()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "config");
            new MemberAccount("FAKEMEMBER", "FAKEMEMBERPASSWORD").Save();

            string path = Path.Combine(_root, "config", MemberAccount.FileName);
            Assert.Equal(UnixFileMode.UserRead | UnixFileMode.UserWrite, File.GetUnixFileMode(path));
            Assert.Equal(("FAKEMEMBER", "FAKEMEMBERPASSWORD"), (MemberAccount.Load().User, MemberAccount.Load().Password));
            Assert.False(File.Exists(path + ".part"));

            new EmuSen.Galaxia.Models.AppSettings().Save();
            Assert.DoesNotContain("FAKEMEMBER", File.ReadAllText(Path.Combine(_root, "config", "appsettings.json")));
        }

        private static void WriteDeveloper(string directory, string id)
        {
            Directory.CreateDirectory(directory);
            File.WriteAllText(Path.Combine(directory, DeveloperCredentials.FileName), JsonSerializer.Serialize(new { devid = id, devpassword = "FAKEPW", softname = "EmuSen-Test" }));
        }

        // --- never in git ---

        private static readonly Regex DevPasswordValue = new(@"(?i)devpassword=([A-Za-z0-9._~%+-]+)");

        private static IReadOnlyList<string> TrackedFiles()
        {
            var git = new ProcessStartInfo("git", "ls-files -z") { WorkingDirectory = ConfigRoot.Directory, RedirectStandardOutput = true, UseShellExecute = false };
            using Process process = Process.Start(git)!;
            string output = process.StandardOutput.ReadToEnd();
            process.WaitForExit();
            Assert.Equal(0, process.ExitCode);
            return output.Split('\0', StringSplitOptions.RemoveEmptyEntries);
        }

        [Fact]
        public void Neither_credential_file_is_tracked_and_the_gitignore_names_both()
        {
            IReadOnlyList<string> tracked = TrackedFiles();
            Assert.NotEmpty(tracked);
            Assert.DoesNotContain(tracked, f => Path.GetFileName(f) is DeveloperCredentials.FileName or MemberAccount.FileName);
            string[] ignored = File.ReadAllLines(Path.Combine(ConfigRoot.Directory, ".gitignore"));
            Assert.Contains(DeveloperCredentials.FileName, ignored);
            Assert.Contains(MemberAccount.FileName, ignored);
        }

        // A value after devpassword= must be a placeholder: one starting FAKE, as the tests spell theirs. The developer's real password, when its file is here, must be nowhere at all.
        [Fact]
        public void No_tracked_file_holds_a_real_devpassword()
        {
            string[] real = RealDeveloperValues();
            var offending = new List<string>();
            foreach (string file in TrackedFiles())
            {
                string path = Path.Combine(ConfigRoot.Directory, file);
                if (!File.Exists(path) || new FileInfo(path).Length > 8 << 20) continue;
                byte[] bytes = File.ReadAllBytes(path);
                string text = Encoding.UTF8.GetString(bytes);
                bool placeholderOnly = DevPasswordValue.Matches(text).All(m => m.Groups[1].Value.StartsWith("FAKE", StringComparison.OrdinalIgnoreCase));
                bool holdsReal = real.Any(r => text.Contains(r, StringComparison.Ordinal) || text.Contains(Uri.EscapeDataString(r), StringComparison.Ordinal));
                if (!placeholderOnly || holdsReal) offending.Add(file);
            }
            Assert.True(offending.Count == 0, "devpassword in: " + string.Join(", ", offending));
        }

        // Read only to be looked for; never handed to anything that sends, embeds or prints it.
        internal static string[] RealDeveloperValues()
        {
            string path = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "EmuSen", DeveloperCredentials.FileName);
            try
            {
                using JsonDocument doc = JsonDocument.Parse(File.ReadAllText(path));
                // The id is not secret and is an ordinary word in tracked files, so only the password is looked for (§4.65).
                return doc.RootElement.TryGetProperty("devpassword", out JsonElement p) && p.GetString() is { Length: >= 4 } s ? [s] : [];
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or JsonException)
            {
                return [];
            }
        }
    }
}
