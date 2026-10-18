using System;
using System.IO;
using System.IO.Compression;
using System.Net.Http;
using System.Net.Security;
using System.Security.Cryptography.X509Certificates;
using EmuSen.Crystal.Services;

namespace EmuSen.Crystal.Cheats
{
    public readonly struct CheatDatabaseInstallResult
    {
        public int Installed { get; init; }

        // Entries that were not .cht, or that tried to write outside the target.
        public int Skipped { get; init; }
    }

    // Downloads to the player's machine on request; EmuSen redistributes nothing - see `man cheat`.
    public static class CheatDatabaseInstaller
    {
        // What RetroArch's own "Update Cheats" pulls.
        public const string LibretroCheatsUrl = "https://buildbot.libretro.com/assets/frontend/cheats.zip";

        // CC BY-SA 4.0 requires attribution, so it goes in front of the player, not in a doc.
        public const string Attribution =
            "Cheat data comes from the libretro cheat database (github.com/libretro/libretro-database),\n" +
            "licensed CC BY-SA 4.0. The codes themselves were collected from community sources,\n" +
            "substantially GameHacking.org - credit belongs to their original authors.\n" +
            "EmuSen ships none of this data; it is downloaded to your machine at your request.";

        // Swapped in tests so the unpack path runs without a network.
        public static Func<string, Stream>? FetchOverride { get; set; }

        // The whole call's limit, redirects and body included.
        public static readonly TimeSpan FetchTimeout = TimeSpan.FromMinutes(5);

        public static Stream Fetch(string url)
        {
            if (FetchOverride is not null) return FetchOverride(url);
            if (!NativeDownload.Active || !ServiceLibrary.Takes(url)) return ManagedFetch(url, FetchTimeout);
            return new MemoryStream(NativeDownload.Fetch(url, FetchTimeout));
        }

        // The C# download: the failover and the parity class's oracle; testRoot is trusted beside the system's store for a test's own server - see EmuSen_Backend.md §16.
        public static Stream ManagedFetch(string url, TimeSpan timeout, X509Certificate2? testRoot = null)
        {
            using var http = testRoot is null ? new HttpClient() : new HttpClient(Trusting(testRoot));
            http.Timeout = timeout;
            // ZipArchive must seek, and an HTTP response stream cannot.
            byte[] bytes = http.GetByteArrayAsync(url).GetAwaiter().GetResult();
            return new MemoryStream(bytes);
        }

        private static SocketsHttpHandler Trusting(X509Certificate2 root)
        {
            var handler = new SocketsHttpHandler();
            handler.SslOptions.RemoteCertificateValidationCallback = (_, certificate, _, errors) =>
            {
                if (errors == SslPolicyErrors.None) return true;
                if (errors != SslPolicyErrors.RemoteCertificateChainErrors || certificate is null) return false;
                using var chain = new X509Chain();
                chain.ChainPolicy.TrustMode = X509ChainTrustMode.CustomRootTrust;
                chain.ChainPolicy.CustomTrustStore.Add(root);
                chain.ChainPolicy.RevocationMode = X509RevocationMode.NoCheck;
                return chain.Build(X509CertificateLoader.LoadCertificate(certificate.GetRawCertData()));
            };
            return handler;
        }

        public static CheatDatabaseInstallResult Install(Stream zip, string targetDirectory)
        {
            if (!CheatsNative.Active || !Services.ServiceLibrary.Takes(targetDirectory)) return ManagedInstall(zip, targetDirectory);
            byte[] bytes;
            using (zip)
            {
                if (zip.CanSeek) zip.Position = 0;
                var copy = new MemoryStream();
                zip.CopyTo(copy);
                bytes = copy.ToArray();
            }
            return CheatsNative.Install(bytes, targetDirectory) ?? ManagedInstall(new MemoryStream(bytes), targetDirectory);
        }

        // The C# install: the failover, an archive of a form the library leaves to it, and the parity class's oracle - see EmuSen_Backend.md §15.3.
        public static CheatDatabaseInstallResult ManagedInstall(Stream zip, string targetDirectory)
        {
            Directory.CreateDirectory(targetDirectory);
            // Without its trailing separator, or root + '/' below would match no entry at all - see EmuSen_Backend.md §15.3.
            string root = Path.TrimEndingDirectorySeparator(Path.GetFullPath(targetDirectory));

            int installed = 0, skipped = 0;
            using var archive = new ZipArchive(zip, ZipArchiveMode.Read);

            foreach (ZipArchiveEntry entry in archive.Entries)
            {
                if (entry.FullName.EndsWith('/') || entry.Length == 0 && entry.Name.Length == 0) continue;
                if (!entry.Name.EndsWith(".cht", StringComparison.OrdinalIgnoreCase)) { skipped++; continue; }

                // Zip-slip: an entry naming ../.. must not write outside the target.
                string destination = Path.GetFullPath(Path.Combine(root, entry.FullName));
                if (!destination.StartsWith(root + Path.DirectorySeparatorChar, StringComparison.Ordinal))
                {
                    skipped++;
                    continue;
                }

                try
                {
                    Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
                    entry.ExtractToFile(destination, overwrite: true);
                    installed++;
                }
                catch
                {
                    skipped++;
                }
            }

            return new CheatDatabaseInstallResult { Installed = installed, Skipped = skipped };
        }
    }
}
