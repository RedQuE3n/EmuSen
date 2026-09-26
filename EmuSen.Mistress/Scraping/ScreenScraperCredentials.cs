using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Galaxia;

namespace EmuSen.Mistress.Scraping
{
    // EmuSen's own developer identity at ScreenScraper, read from a 0600 file outside every repository and never shipped - see EmuSen_BigPicture.md §5.7 and §17.
    public sealed class DeveloperCredentials
    {
        public const string FileName = "screenscraper-developer.json";

        public DeveloperCredentials(string devId, string devPassword, string softName)
        {
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

        // Never the id or password, so a log line or an exception holding this object says nothing it should not.
        public override string ToString() => $"ScreenScraper developer credentials for {SoftName}";

        // The config directory first, then ~/.config/EmuSen where the file was put; a test that moved the config directory never reaches the real one.
        public static IEnumerable<string> Locations()
        {
            yield return ConfigStore.For(FileName);
            if (ConfigStore.OverrideDirectory is not null && ConfigStore.OverrideLegacyDirectory is null) yield break;
            yield return Path.Combine(ConfigStore.LegacyDirectory, FileName);
        }

        public static DeveloperCredentials? Load()
        {
            foreach (string path in Locations())
                if (Read(path) is { } found) return found;
            return null;
        }

        // Null for a missing or unreadable file, or one without all three fields; the reason is never the file's contents.
        public static DeveloperCredentials? Read(string path)
        {
            try
            {
                if (!File.Exists(path)) return null;
                JsonNode? root = JsonNode.Parse(File.ReadAllText(path));
                string? id = root?["devid"]?.GetValue<string>(), password = root?["devpassword"]?.GetValue<string>(), soft = root?["softname"]?.GetValue<string>();
                if (string.IsNullOrWhiteSpace(id) || string.IsNullOrWhiteSpace(password) || string.IsNullOrWhiteSpace(soft)) return null;
                return new DeveloperCredentials(id, password, soft);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or JsonException or InvalidOperationException or FormatException)
            {
                return null;
            }
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

        public bool IsSet => User.Length > 0 && Password.Length > 0;

        public override string ToString() => IsSet ? "a ScreenScraper member account" : "no ScreenScraper member account";

        public static string PathOf => ConfigStore.For(FileName);

        public static MemberAccount Load()
        {
            try
            {
                if (!File.Exists(PathOf)) return new MemberAccount("", "");
                JsonNode? root = JsonNode.Parse(File.ReadAllText(PathOf));
                return new MemberAccount(root?["ssid"]?.GetValue<string>() ?? "", root?["sspassword"]?.GetValue<string>() ?? "");
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
            using (var stream = new FileStream(temp, options))
                JsonSerializer.Serialize(stream, new JsonObject { ["ssid"] = User, ["sspassword"] = Password });
            if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(temp, UnixFileMode.UserRead | UnixFileMode.UserWrite);
            File.Move(temp, path, overwrite: true);
        }
    }
}
