using System.IO.Compression;
using System.Net;
using System.Net.Security;
using System.Net.Sockets;
using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;
using System.Text;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Services;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The cheat database's download on emusen-net held to the C# HttpClient against an HTTPS server of the test's own, certificates made at test time; never the internet - see EmuSen_Backend.md §16.
    [Collection(TestCollections.ProcessGlobals)]
    public class CheatDownloadParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDownloadParity_" + Guid.NewGuid().ToString("N"));
        private static readonly TimeSpan Usual = TimeSpan.FromSeconds(20);

        // Every exception an update threw, whole, for a failure's message: Thrown keeps to the depth both sides compare at.
        private readonly List<string> _thrown = new();

        private static readonly Lazy<Certs> Made = new(Certs.Make);

        public CheatDownloadParityTests()
        {
            Assert.True(ServiceLibrary.Ready, ServiceLibrary.Report);
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private sealed class Certs
        {
            public required X509Certificate2 Root, Leaf, Stranger, OtherName;

            public static Certs Make()
            {
                using RSA rootKey = RSA.Create(2048);
                var rootRequest = new CertificateRequest("CN=EmuSen Test Root", rootKey, HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1);
                rootRequest.CertificateExtensions.Add(new X509BasicConstraintsExtension(true, false, 0, true));
                rootRequest.CertificateExtensions.Add(new X509KeyUsageExtension(X509KeyUsageFlags.KeyCertSign | X509KeyUsageFlags.CrlSign, true));
                DateTimeOffset now = DateTimeOffset.UtcNow;
                X509Certificate2 root = rootRequest.CreateSelfSigned(now.AddDays(-1), now.AddDays(30));
                return new Certs
                {
                    Root = X509CertificateLoader.LoadCertificate(root.RawData),
                    Leaf = Server("localhost", root),
                    Stranger = Server("localhost", null),
                    OtherName = Server("other.test", root),
                };
            }

            private static X509Certificate2 Server(string name, X509Certificate2? issuer)
            {
                using RSA key = RSA.Create(2048);
                var request = new CertificateRequest($"CN={name}", key, HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1);
                var names = new SubjectAlternativeNameBuilder();
                names.AddDnsName(name);
                if (name == "localhost") names.AddIpAddress(IPAddress.Loopback);
                request.CertificateExtensions.Add(names.Build());
                request.CertificateExtensions.Add(new X509EnhancedKeyUsageExtension(new OidCollection { new Oid("1.3.6.1.5.5.7.3.1") }, false));
                DateTimeOffset now = DateTimeOffset.UtcNow;
                X509Certificate2 made = issuer is null
                    ? request.CreateSelfSigned(now.AddDays(-1), now.AddDays(30))
                    : request.Create(issuer, now.AddDays(-1), now.AddDays(29), RandomNumberGenerator.GetBytes(8)).CopyWithPrivateKey(key);
                return X509CertificateLoader.LoadPkcs12(made.Export(X509ContentType.Pfx), null);
            }
        }

        // An HTTPS server on 127.0.0.1 handing each request's path and stream to the scenario, closing the connection after it.
        private sealed class Server : IDisposable
        {
            private readonly TcpListener _listener = new(IPAddress.Loopback, 0);
            private readonly X509Certificate2 _cert;
            private readonly Func<string, Stream, Task> _handler;
            public readonly List<string> Heads = new();
            public int Connections;

            // What the server itself swallowed, for a failure's message only: a parity comparison never reads it.
            public readonly List<string> Faults = new();

            public Server(X509Certificate2 cert, Func<string, Stream, Task> handler)
            {
                _cert = cert;
                _handler = handler;
                _listener.Start();
                _ = Task.Run(Accept);
            }

            public int Port => ((IPEndPoint)_listener.LocalEndpoint).Port;

            public string Url(string path) => $"https://localhost:{Port}{path}";

            private async Task Accept()
            {
                while (true)
                {
                    TcpClient client;
                    try { client = await _listener.AcceptTcpClientAsync(); }
                    catch (Exception e)
                    {
                        lock (Faults) Faults.Add("accept: " + e);
                        return;
                    }
                    Interlocked.Increment(ref Connections);
                    _ = Task.Run(() => Serve(client));
                }
            }

            private async Task Serve(TcpClient client)
            {
                using (client)
                {
                    try
                    {
                        var tls = new SslStream(client.GetStream());
                        await tls.AuthenticateAsServerAsync(_cert);
                        var head = new List<byte>();
                        var one = new byte[1];
                        while (head.Count < 4 || head[^4] != '\r' || head[^3] != '\n' || head[^2] != '\r' || head[^1] != '\n')
                        {
                            if (await tls.ReadAsync(one) == 0) return;
                            head.Add(one[0]);
                        }
                        string text = Encoding.ASCII.GetString(head.ToArray());
                        lock (Heads) Heads.Add(text);
                        await _handler(text.Split(' ')[1], tls);
                        await tls.FlushAsync();
                    }
                    catch (Exception e) { lock (Faults) Faults.Add("serve: " + e); }
                }
            }

            // The requests seen, header names in lower case, since .NET writes `Host` and ureq `host`.
            public string Seen()
            {
                lock (Heads)
                    return $"{Heads.Count} request(s) on {Connections} connection(s)\n" + string.Join("", Heads.Select(h =>
                        string.Join("\r\n", h.Split("\r\n").Select((line, i) => i == 0 || !line.Contains(':') ? line : line[..line.IndexOf(':')].ToLowerInvariant() + line[line.IndexOf(':')..]))));
            }

            public void Clear()
            {
                lock (Heads) Heads.Clear();
                Connections = 0;
            }

            public void Dispose() => _listener.Stop();
        }

        private static Func<string, Stream, Task> Reply(string head, byte[]? body = null, int? send = null) => async (_, s) =>
        {
            await s.WriteAsync(Encoding.ASCII.GetBytes(head));
            if (body is not null) await s.WriteAsync(body.AsMemory(0, send ?? body.Length));
        };

        private static byte[] Archive()
        {
            var bytes = new MemoryStream();
            using (var zip = new ZipArchive(bytes, ZipArchiveMode.Create, leaveOpen: true))
            {
                foreach ((string name, string text) in new[]
                {
                    ("cht/Nintendo - Super Nintendo Entertainment System/Super Mario World (USA).cht", "cheats = 1\ncheat0_desc = \"Lives\"\ncheat0_code = \"7E0DBE:63\"\ncheat0_enable = false\n"),
                    ("cht/Sega - Mega Drive - Genesis/Sonic The Hedgehog (USA, Europe).cht", "cheats = 1\ncheat0_desc = \"Rings\"\ncheat0_code = \"FFFE20:0063\"\ncheat0_enable = false\n"),
                    ("cht/README.txt", "not a cheat"),
                })
                {
                    using Stream e = zip.CreateEntry(name).Open();
                    e.Write(Encoding.UTF8.GetBytes(text));
                }
            }
            return bytes.ToArray();
        }

        private static string Thrown(Exception e)
        {
            string text = $"threw {e.GetType().FullName}: {e.Message}";
            return e.InnerException is { } inner ? text + $"\n  inner {inner.GetType().FullName}: {inner.Message}" : text;
        }

        // What an update leaves: what it said, and every file it wrote with its bytes' hash.
        private string Update(Func<Stream> fetch, Func<byte[], string, CheatDatabaseInstallResult> install, string name)
        {
            string target = Path.Combine(_root, name);
            string said;
            try
            {
                byte[] bytes;
                using (Stream s = fetch()) { var copy = new MemoryStream(); s.CopyTo(copy); bytes = copy.ToArray(); }
                CheatDatabaseInstallResult r = install(bytes, target);
                said = $"installed {r.Installed}, skipped {r.Skipped}";
            }
            catch (Exception e)
            {
                said = Thrown(e);
                _thrown.Add($"{name}: {e}");
            }
            var files = Directory.Exists(target)
                ? Directory.EnumerateFiles(target, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                    .Select(f => $"  {Path.GetRelativePath(target, f)} {Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(f)))[..16]}")
                : new[] { "  no folder" };
            return said + "\n" + string.Join("\n", files);
        }

        private string Both(Server server, string path, TimeSpan? timeout = null, bool trusted = true)
        {
            Certs c = Made.Value;
            TimeSpan t = timeout ?? Usual;
            string url = server.Url(path);
            string id = Guid.NewGuid().ToString("N")[..8];
            server.Clear();
            string managed = Update(() => CheatDatabaseInstaller.ManagedFetch(url, t, trusted ? c.Root : null), (b, d) => CheatDatabaseInstaller.ManagedInstall(new MemoryStream(b), d), "m" + id) + "\n" + server.Seen();
            server.Clear();
            string native = Update(() => new MemoryStream(NativeDownload.Fetch(url, t, trusted ? c.Root.ExportCertificatePem() : null)),
                (b, d) => CheatsNative.Install(b, d) ?? CheatDatabaseInstaller.ManagedInstall(new MemoryStream(b), d), "n" + id) + "\n" + server.Seen();
            native = native.Replace(" n" + id, " m" + id);
            if (managed != native)
            {
                string faults;
                lock (server.Faults) faults = string.Join("\n", server.Faults);
                Assert.Fail($"C#:\n{managed}\n\nRust:\n{native}\n\nThrown:\n{string.Join("\n", _thrown)}\n\nThe server swallowed:\n{faults}");
            }
            return managed;
        }

        [Fact]
        public void An_archive_installs_alike_whole_chunked_and_after_a_redirect()
        {
            byte[] zip = Archive();
            using var whole = new Server(Made.Value.Leaf, Reply($"HTTP/1.1 200 OK\r\nContent-Length: {zip.Length}\r\n\r\n", zip));
            Assert.StartsWith("installed 2, skipped 1", Both(whole, "/assets/frontend/cheats.zip"));

            using var chunked = new Server(Made.Value.Leaf, async (_, s) =>
            {
                await s.WriteAsync(Encoding.ASCII.GetBytes("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n"));
                for (int at = 0; at < zip.Length; at += 100)
                {
                    int n = Math.Min(100, zip.Length - at);
                    await s.WriteAsync(Encoding.ASCII.GetBytes($"{n:x}\r\n"));
                    await s.WriteAsync(zip.AsMemory(at, n));
                    await s.WriteAsync("\r\n"u8.ToArray());
                }
                await s.WriteAsync("0\r\n\r\n"u8.ToArray());
            });
            Assert.StartsWith("installed 2", Both(chunked, "/"));

            using var redirected = new Server(Made.Value.Leaf, (path, s) => path.StartsWith("/cheats.zip?", StringComparison.Ordinal)
                ? Reply($"HTTP/1.1 200 OK\r\nContent-Length: {zip.Length}\r\n\r\n", zip)(path, s)
                : Reply("HTTP/1.1 302 Found\r\nLocation: ../cheats.zip?from=1\r\nContent-Length: 0\r\n\r\n")(path, s));
            Assert.Contains("2 request(s)", Both(redirected, "/frontend/start"));
        }

        [Fact]
        public void Redirects_are_refused_alike_past_fifty_and_from_https_to_http()
        {
            using var looping = new Server(Made.Value.Leaf, Reply("HTTP/1.1 301 Moved Permanently\r\nLocation: /again\r\nContent-Length: 0\r\n\r\n"));
            Assert.Contains("51 request(s)", Both(looping, "/"));
            using var downgrade = new Server(Made.Value.Leaf, Reply("HTTP/1.1 302 Found\r\nLocation: http://localhost:1/cheats.zip\r\nContent-Length: 0\r\n\r\n"));
            Assert.StartsWith("threw System.Net.Http.HttpRequestException: Response status code does not indicate success: 302 (Found).", Both(downgrade, "/"));
        }

        [Fact]
        public void A_status_that_is_not_success_is_refused_alike_with_the_reason_sent()
        {
            foreach (string line in new[] { "404 Not Found", "500 Internal Server Error", "404 Nope Nope", "404 ", "503 Service Unavailable" })
            {
                using var server = new Server(Made.Value.Leaf, Reply($"HTTP/1.1 {line}\r\nContent-Length: 3\r\n\r\n", "no!"u8.ToArray()));
                Assert.Contains("no folder", Both(server, "/"));
            }
        }

        [Fact]
        public void A_connection_cut_halfway_installs_nothing_alike()
        {
            byte[] zip = Archive();
            using var counted = new Server(Made.Value.Leaf, Reply($"HTTP/1.1 200 OK\r\nContent-Length: {zip.Length}\r\n\r\n", zip, zip.Length / 2));
            Assert.StartsWith("threw System.Net.Http.HttpRequestException: Error while copying content to a stream.", Both(counted, "/"));
            using var chunked = new Server(Made.Value.Leaf, async (_, s) =>
            {
                await s.WriteAsync(Encoding.ASCII.GetBytes($"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{zip.Length:x}\r\n"));
                await s.WriteAsync(zip.AsMemory(0, zip.Length / 2));
            });
            Assert.Contains("no folder", Both(chunked, "/"));
            using var closed = new Server(Made.Value.Leaf, Reply("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n", zip, zip.Length / 2));
            Assert.Contains("InvalidDataException", Both(closed, "/"));
        }

        [Fact]
        public void A_corrupt_archive_is_refused_alike()
        {
            byte[] junk = Encoding.ASCII.GetBytes(new string('x', 5000));
            using var server = new Server(Made.Value.Leaf, Reply($"HTTP/1.1 200 OK\r\nContent-Length: {junk.Length}\r\n\r\n", junk));
            Assert.StartsWith("threw System.IO.InvalidDataException", Both(server, "/"));
        }

        [Fact]
        public void A_slow_server_and_a_silent_one_reach_the_timeout_alike()
        {
            using var slow = new Server(Made.Value.Leaf, async (_, s) =>
            {
                await s.WriteAsync("HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n0123456789"u8.ToArray());
                await s.FlushAsync();
                await Task.Delay(5000);
            });
            Assert.StartsWith("threw System.Threading.Tasks.TaskCanceledException: The request was canceled due to the configured HttpClient.Timeout of 1.5 seconds elapsing.", Both(slow, "/", TimeSpan.FromSeconds(1.5)));
            using var silent = new Server(Made.Value.Leaf, (_, _) => Task.Delay(5000));
            Assert.Contains("TaskCanceledException", Both(silent, "/", TimeSpan.FromSeconds(1.5)));
            using var hops = new Server(Made.Value.Leaf, async (path, s) =>
            {
                await Task.Delay(1000);
                await Reply(path == "/start" ? "HTTP/1.1 302 Found\r\nLocation: /next\r\nContent-Length: 0\r\n\r\n" : "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nhi")(path, s);
            });
            Assert.Contains("TaskCanceledException", Both(hops, "/start", TimeSpan.FromSeconds(1.5)));
        }

        [Fact]
        public void A_certificate_the_system_does_not_trust_is_refused_alike_with_the_words()
        {
            using var stranger = new Server(Made.Value.Stranger, Reply("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"));
            string shown = Both(stranger, "/", trusted: false);
            Assert.StartsWith("threw System.Net.Http.HttpRequestException: The SSL connection could not be established, see inner exception.", shown);
            Assert.Contains("0 request(s)", shown);
            using var ours = new Server(Made.Value.Leaf, Reply("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"));
            Assert.Contains("PartialChain", Both(ours, "/", trusted: false));
            using var misnamed = new Server(Made.Value.OtherName, Reply("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"));
            Assert.Contains("RemoteCertificateNameMismatch, RemoteCertificateChainErrors", Both(misnamed, "/", trusted: false));
        }

        // A trusted chain for another name: the C# oracle's test trust answers in its own words, so the Rust is held to .NET's, measured with the root as the system's store on 2026-10-18.
        [Fact]
        public void A_trusted_certificate_for_another_name_is_refused_in_dotnets_words()
        {
            using var misnamed = new Server(Made.Value.OtherName, Reply("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"));
            var thrown = Assert.Throws<HttpRequestException>(() => NativeDownload.Fetch(misnamed.Url("/"), Usual, Made.Value.Root.ExportCertificatePem()));
            Assert.Equal("The SSL connection could not be established, see inner exception.", thrown.Message);
            Assert.Equal("The remote certificate is invalid according to the validation procedure: RemoteCertificateNameMismatch", thrown.InnerException!.Message);
            Assert.Throws<HttpRequestException>(() => CheatDatabaseInstaller.ManagedFetch(misnamed.Url("/"), Usual, Made.Value.Root));
            Assert.Empty(misnamed.Heads);
        }

        [Fact]
        public void What_is_not_http_and_a_request_no_byte_answers_fail_alike()
        {
            using var garbage = new Server(Made.Value.Leaf, Reply("SSH-2.0-OpenSSH\r\n\r\n"));
            Assert.StartsWith("threw System.Net.Http.HttpRequestException: Received an invalid status line: 'SSH-2.0-OpenSSH'.", Both(garbage, "/"));
            using var unanswered = new Server(Made.Value.Leaf, (_, _) => Task.CompletedTask);
            Assert.Contains("4 request(s) on 4 connection(s)", Both(unanswered, "/"));
        }

        [Fact]
        public void A_refused_connection_fails_alike_naming_its_place()
        {
            var listener = new TcpListener(IPAddress.Loopback, 0);
            listener.Start();
            int port = ((IPEndPoint)listener.LocalEndpoint).Port;
            listener.Stop();
            string url = $"https://localhost:{port}/";
            string managed, native;
            try { CheatDatabaseInstaller.ManagedFetch(url, Usual); managed = "fetched"; } catch (Exception e) { managed = Thrown(e); }
            try { NativeDownload.Fetch(url, Usual); native = "fetched"; } catch (Exception e) { native = Thrown(e); }
            Assert.Equal($"threw System.Net.Http.HttpRequestException: Connection refused (localhost:{port})\n  inner System.Net.Sockets.SocketException: Connection refused", managed);
            Assert.Equal(managed, native);
        }

        [Fact]
        public void Progress_is_drained_to_the_body_and_a_cancel_ends_the_download_at_once()
        {
            byte[] zip = Archive();
            using var whole = new Server(Made.Value.Leaf, Reply($"HTTP/1.1 200 OK\r\nContent-Length: {zip.Length}\r\n\r\n", zip));
            using (var d = new NativeDownload(whole.Url("/"), Usual, Made.Value.Root.ExportCertificatePem()))
            {
                Assert.Equal(NativeDownload.Done, d.Wait(Usual));
                Assert.Equal((NativeDownload.Done, (long)zip.Length, (long)zip.Length), d.Progress());
                Assert.Equal(zip, d.Body());
            }

            using var dripping = new Server(Made.Value.Leaf, async (_, s) =>
            {
                await s.WriteAsync("HTTP/1.1 200 OK\r\nContent-Length: 100000\r\n\r\n"u8.ToArray());
                await s.WriteAsync(new byte[600]);
                await s.FlushAsync();
                await Task.Delay(10000);
            });
            using (var d = new NativeDownload(dripping.Url("/"), Usual, Made.Value.Root.ExportCertificatePem()))
            {
                long last = 0;
                var started = DateTime.UtcNow;
                while (d.Progress().Received < 500)
                {
                    (int state, long received, _) = d.Progress();
                    Assert.Equal(NativeDownload.Running, state);
                    Assert.True(received >= last);
                    last = received;
                    Assert.True(DateTime.UtcNow - started < Usual);
                    Thread.Sleep(10);
                }
                Assert.Equal(100000, d.Progress().Total);
                var asked = DateTime.UtcNow;
                d.Cancel();
                Assert.Equal(NativeDownload.Cancelled, d.Wait(TimeSpan.FromSeconds(5)));
                Assert.True(DateTime.UtcNow - asked < TimeSpan.FromMilliseconds(200));
                var thrown = Assert.Throws<OperationCanceledException>(() => d.Body());
                Assert.Equal("The operation was canceled.", thrown.Message);
            }
        }

        [Fact]
        public void The_public_fetch_takes_the_switch_and_a_bad_request_is_refused()
        {
            Assert.Equal("https://buildbot.libretro.com/assets/frontend/cheats.zip", CheatDatabaseInstaller.LibretroCheatsUrl);
            Assert.Equal(TimeSpan.FromMinutes(5), CheatDatabaseInstaller.FetchTimeout);
            var fault = Assert.Throws<ServiceFault>(() => new NativeDownload("https://x/", Usual, "-----BEGIN CERTIFICATE-----\n!!\n-----END CERTIFICATE-----\n"));
            Assert.Equal(ServiceFault.Parse, fault.Status);
            Assert.Throws<NotSupportedException>(() => NativeDownload.Fetch("ftp://example.org/", Usual));
        }
    }
}
