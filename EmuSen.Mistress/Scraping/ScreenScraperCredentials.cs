using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Galaxia;

namespace EmuSen.Mistress.Scraping
{
    // Where the developer credentials in use came from; the embedded ones are never written anywhere or shown.
    public enum DeveloperOrigin { TreeFile, UserFile, Embedded }

    // EmuSen's own developer identity at ScreenScraper: a 0600 file outside every repository, else what a published build carries - see EmuSen_Settings_Reference.md §4.65.
    public sealed class DeveloperCredentials
    {
        public const string FileName = "screenscraper-developer.json";

        // The manifest resource ScreenScraperDeveloper.targets embeds at publish; must match EmuSenScreenScraperResource there.
        public const string ResourceName = "EmuSen.Mistress.Scraping.Developer";

        public const string NoneHere = "this build carries no developer credentials and EmuSen's developer file is not on this computer";

        // Replaced by tests with a fake blob; the default reads this assembly's own resource, which only a publish puts there.
        internal static Func<Stream?> EmbeddedSource = () => typeof(DeveloperCredentials).Assembly.GetManifestResourceStream(ResourceName);

        public DeveloperCredentials(string devId, string devPassword, string softName, DeveloperOrigin origin = DeveloperOrigin.TreeFile)
        {
            Origin = origin;
            DevId = devId;
            DevPassword = devPassword;
            SoftName = softName;
            ScrapeRedactor.Register(devId);
            ScrapeRedactor.Register(devPassword);
            ScrapeRedactor.Register(Uri.EscapeDataString(devPassword));
        }

        public string DevId { get; }
        public string DevPassword { get; }
        public string SoftName { get; }
        public DeveloperOrigin Origin { get; }

        // Never the id or password, so a log line or an exception holding this object says nothing it should not.
        public override string ToString() => $"ScreenScraper developer credentials for {SoftName}";

        // The config directory first, then ~/.config/EmuSen where the file was put; a test that moved the config directory never reaches the real one.
        public static IEnumerable<string> Locations()
        {
            yield return ConfigStore.For(FileName);
            if (ConfigStore.OverrideDirectory is not null && ConfigStore.OverrideLegacyDirectory is null) yield break;
            yield return Path.Combine(ConfigStore.LegacyDirectory, FileName);
        }

        // The tree's own file, then ~/.config/EmuSen's, then the build's embedded credentials.
        public static DeveloperCredentials? Load()
        {
            DeveloperOrigin origin = DeveloperOrigin.TreeFile;
            foreach (string path in Locations())
            {
                if (Read(path, origin) is { } found) return found;
                origin = DeveloperOrigin.UserFile;
            }
            return Embedded();
        }

        // Null for a missing or unreadable file, or one without all three fields; the reason is never the file's contents.
        public static DeveloperCredentials? Read(string path, DeveloperOrigin origin = DeveloperOrigin.TreeFile)
        {
            try
            {
                return File.Exists(path) ? Parse(File.ReadAllText(path), origin) : null;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or JsonException or InvalidOperationException or FormatException)
            {
                return null;
            }
        }

        public static DeveloperCredentials? Embedded()
        {
            try
            {
                using Stream? stream = EmbeddedSource();
                if (stream is null) return null;
                using var memory = new MemoryStream();
                stream.CopyTo(memory);
                return Decode(memory.ToArray());
            }
            catch (IOException)
            {
                return null;
            }
        }

        // What ScreenScraperDeveloper.targets writes: "ESSD", a version, the key's length, the key, then the JSON XORed with it; null for anything else.
        public static DeveloperCredentials? Decode(byte[] blob)
        {
            try
            {
                int keyLength = blob.Length > 5 ? blob[5] : 0;
                if (blob.Length <= 6 + keyLength || keyLength == 0 || blob[0] != 'E' || blob[1] != 'S' || blob[2] != 'S' || blob[3] != 'D' || blob[4] != 1) return null;
                byte[] body = new byte[blob.Length - 6 - keyLength];
                for (int i = 0; i < body.Length; i++) body[i] = (byte)(blob[6 + keyLength + i] ^ blob[6 + i % keyLength]);
                try { return Parse(System.Text.Encoding.UTF8.GetString(body), DeveloperOrigin.Embedded); }
                finally { Array.Clear(body); }
            }
            catch (Exception ex) when (ex is JsonException or InvalidOperationException or FormatException or ArgumentException)
            {
                return null;
            }
        }

        private static DeveloperCredentials? Parse(string json, DeveloperOrigin origin)
        {
            JsonNode? root = JsonNode.Parse(json);
            string? id = root?["devid"]?.GetValue<string>(), password = root?["devpassword"]?.GetValue<string>(), soft = root?["softname"]?.GetValue<string>();
            if (string.IsNullOrWhiteSpace(id) || string.IsNullOrWhiteSpace(password) || string.IsNullOrWhiteSpace(soft)) return null;
            return new DeveloperCredentials(id, password, soft, origin);
        }
    }

    // The player's own ScreenScraper member account, typed in Preferences and kept in its own 0600 file, never in appsettings.json - see EmuSen_BigPicture.md §5.7.
    public sealed class MemberAccount
    {
        public const string FileName = "screenscraper.json";

        public MemberAccount(string user, string password)
        {
            User = user;
            Password = password;
            ScrapeRedactor.Register(user);
            ScrapeRedactor.Register(password);
            ScrapeRedactor.Register(Uri.EscapeDataString(password));
        }

        public string User { get; }
        public string Password { get; }

        // When ScreenScraper last accepted it through Log In or Check; null for a file the old two boxes wrote, shown as not checked.
        public DateTime? Verified { get; init; }

        public bool IsSet => User.Length > 0 && Password.Length > 0;

        public override string ToString() => IsSet ? "a ScreenScraper member account" : "no ScreenScraper member account";

        public static string PathOf => ConfigStore.For(FileName);

        public static MemberAccount Load()
        {
            try
            {
                if (!File.Exists(PathOf)) return new MemberAccount("", "");
                JsonNode? root = JsonNode.Parse(File.ReadAllText(PathOf));
                DateTime? verified = DateTime.TryParse(root?["verified"]?.GetValue<string>(), System.Globalization.CultureInfo.InvariantCulture,
                    System.Globalization.DateTimeStyles.AdjustToUniversal | System.Globalization.DateTimeStyles.AssumeUniversal, out DateTime at) ? at : null;
                return new MemberAccount(root?["ssid"]?.GetValue<string>() ?? "", root?["sspassword"]?.GetValue<string>() ?? "") { Verified = verified };
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or JsonException or InvalidOperationException or FormatException)
            {
                return new MemberAccount("", "");
            }
        }

        // Written whole beside its final name with mode 0600 from the moment it exists, then moved over the old file.
        public void Save()
        {
            string path = PathOf;
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            string temp = path + ".part";
            var options = new FileStreamOptions { Mode = FileMode.Create, Access = FileAccess.Write, Share = FileShare.None };
            if (!OperatingSystem.IsWindows()) options.UnixCreateMode = UnixFileMode.UserRead | UnixFileMode.UserWrite;
            var body = new JsonObject { ["ssid"] = User, ["sspassword"] = Password };
            if (Verified is DateTime at) body["verified"] = at.ToUniversalTime().ToString("O", System.Globalization.CultureInfo.InvariantCulture);
            using (var stream = new FileStream(temp, options))
                JsonSerializer.Serialize(stream, body);
            if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(temp, UnixFileMode.UserRead | UnixFileMode.UserWrite);
            File.Move(temp, path, overwrite: true);
        }

        // Log Out: the file is deleted, not emptied; true when there was one.
        public static bool Delete()
        {
            string path = PathOf;
            if (!File.Exists(path)) return false;
            File.Delete(path);
            return true;
        }
    }
}
