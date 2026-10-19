using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Net.Security;
using System.Net.Sockets;
using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // An HTTPS server on 127.0.0.1 that answers at the real addresses (ScreenScraper's, libretro's, GitHub's) from a handler the test gives, recording every request as it came, so two clients' requests compare byte for byte - see EmuSen_Backend.md §30.7.
    public sealed class ReplayServer : IDisposable
    {
        public static readonly string[] Hosts = ["api.screenscraper.fr", "neoclone.screenscraper.fr", "thumbnails.libretro.com", "api.github.com", "github.com", "objects.githubusercontent.com", "localhost"];

        private static readonly Lazy<(X509Certificate2 Root, X509Certificate2 Leaf)> Made = new(Make);

        public static X509Certificate2 Root => Made.Value.Root;

        public static string RootPem => Made.Value.Root.ExportCertificatePem();

        private static (X509Certificate2, X509Certificate2) Make()
        {
            using RSA rootKey = RSA.Create(2048);
            var rootRequest = new CertificateRequest("CN=EmuSen Replay Root", rootKey, HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1);
            rootRequest.CertificateExtensions.Add(new X509BasicConstraintsExtension(true, false, 0, true));
            rootRequest.CertificateExtensions.Add(new X509KeyUsageExtension(X509KeyUsageFlags.KeyCertSign | X509KeyUsageFlags.CrlSign, true));
            DateTimeOffset now = DateTimeOffset.UtcNow;
            X509Certificate2 root = rootRequest.CreateSelfSigned(now.AddDays(-1), now.AddDays(30));
            using RSA key = RSA.Create(2048);
            var request = new CertificateRequest("CN=replay", key, HashAlgorithmName.SHA256, RSASignaturePadding.Pkcs1);
            var names = new SubjectAlternativeNameBuilder();
            foreach (string host in Hosts) names.AddDnsName(host);
            names.AddIpAddress(IPAddress.Loopback);
            request.CertificateExtensions.Add(names.Build());
            request.CertificateExtensions.Add(new X509EnhancedKeyUsageExtension(new OidCollection { new Oid("1.3.6.1.5.5.7.3.1") }, false));
            X509Certificate2 leaf = request.Create(root, now.AddDays(-1), now.AddDays(29), RandomNumberGenerator.GetBytes(8)).CopyWithPrivateKey(key);
            return (X509CertificateLoader.LoadCertificate(root.RawData), X509CertificateLoader.LoadPkcs12(leaf.Export(X509ContentType.Pfx), null));
        }

        private readonly TcpListener _listener = new(IPAddress.Loopback, 0);
        private readonly HttpMessageInvoker _invoker;
        private readonly List<string> _heads = new();
        public readonly List<string> Faults = new();

        public ReplayServer(HttpMessageHandler handler)
        {
            _invoker = new HttpMessageInvoker(handler, disposeHandler: false);
            _listener.Start();
            _ = Task.Run(Accept);
        }

        public int Port => ((IPEndPoint)_listener.LocalEndpoint).Port;

        public string Address => $"127.0.0.1:{Port}";

        // Every request's head as it came, in order.
        public IReadOnlyList<string> Heads
        {
            get { lock (_heads) return _heads.ToArray(); }
        }

        public void Clear()
        {
            lock (_heads) _heads.Clear();
        }

        // A client as Mistress makes one, its every connection sent here and the test's root trusted.
        public HttpClient Client(string userAgent)
        {
            var handler = new SocketsHttpHandler
            {
                ConnectCallback = async (_, token) =>
                {
                    var socket = new Socket(SocketType.Stream, ProtocolType.Tcp) { NoDelay = true };
                    await socket.ConnectAsync(IPAddress.Loopback, Port, token);
                    return new NetworkStream(socket, ownsSocket: true);
                },
                SslOptions = new SslClientAuthenticationOptions { RemoteCertificateValidationCallback = Trusted },
            };
            var http = new HttpClient(handler) { Timeout = TimeSpan.FromSeconds(30) };
            http.DefaultRequestHeaders.UserAgent.ParseAdd(userAgent);
            return http;
        }

        private static bool Trusted(object sender, X509Certificate? certificate, X509Chain? _, SslPolicyErrors errors)
        {
            if (certificate is null || (errors & SslPolicyErrors.RemoteCertificateNameMismatch) != 0) return false;
            using var chain = new X509Chain();
            chain.ChainPolicy.TrustMode = X509ChainTrustMode.CustomRootTrust;
            chain.ChainPolicy.CustomTrustStore.Add(Root);
            chain.ChainPolicy.RevocationMode = X509RevocationMode.NoCheck;
            return chain.Build(X509CertificateLoader.LoadCertificate(certificate.GetRawCertData()));
        }

        private async Task Accept()
        {
            while (true)
            {
                TcpClient client;
                try { client = await _listener.AcceptTcpClientAsync(); }
                catch (Exception) { return; }
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
                    await tls.AuthenticateAsServerAsync(Made.Value.Leaf);
                    var head = new List<byte>();
                    var one = new byte[1];
                    while (head.Count < 4 || head[^4] != '\r' || head[^3] != '\n' || head[^2] != '\r' || head[^1] != '\n')
                    {
                        if (await tls.ReadAsync(one) == 0) return;
                        head.Add(one[0]);
                    }
                    string text = Encoding.Latin1.GetString(head.ToArray());
                    lock (_heads) _heads.Add(text);
                    string[] lines = text.Split("\r\n");
                    string target = lines[0].Split(' ')[1];
                    string host = lines.Skip(1).Select(l => l.Split(':', 2)).Where(p => p.Length == 2 && p[0].Trim().Equals("host", StringComparison.OrdinalIgnoreCase)).Select(p => p[1].Trim()).FirstOrDefault() ?? "localhost";
                    using var request = new HttpRequestMessage(HttpMethod.Get, $"https://{host}{target}");
                    using HttpResponseMessage response = await _invoker.SendAsync(request, CancellationToken.None);
                    byte[] body = await response.Content.ReadAsByteArrayAsync();
                    var reply = new StringBuilder($"HTTP/1.1 {(int)response.StatusCode} {response.ReasonPhrase ?? ""}\r\n");
                    if (response.Content.Headers.ContentType is { } type) reply.Append($"Content-Type: {type}\r\n");
                    reply.Append($"Content-Length: {body.Length}\r\nConnection: close\r\n\r\n");
                    await tls.WriteAsync(Encoding.ASCII.GetBytes(reply.ToString()));
                    await tls.WriteAsync(body);
                    await tls.FlushAsync();
                }
                catch (Exception e) { lock (Faults) Faults.Add(e.GetType().Name + ": " + e.Message); }
            }
        }

        public void Dispose() => _listener.Stop();
    }
}
