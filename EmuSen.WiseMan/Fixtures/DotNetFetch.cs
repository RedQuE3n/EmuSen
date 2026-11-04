using System;
using System.IO;
using System.Net.Http;
using System.Net.Security;
using System.Security.Cryptography.X509Certificates;

namespace EmuSen.WiseMan.Fixtures
{
    // A download as .NET's HttpClient does it, the oracle emusen-net is held to since the C# download was deleted on 2026-11-04; testRoot is trusted beside the system's store - see EmuSen_Backend.md §16, §40.1.
    public static class DotNetFetch
    {
        public static Stream Fetch(string url, TimeSpan timeout, X509Certificate2? testRoot = null)
        {
            using var http = testRoot is null ? new HttpClient() : new HttpClient(Trusting(testRoot));
            http.Timeout = timeout;
            return new MemoryStream(http.GetByteArrayAsync(url).GetAwaiter().GetResult());
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
    }
}
