using System;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Net.Sockets;
using System.Security.Authentication;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Services
{
    // One download on emusen-net, the platform's HTTP client, under the file services' switch; it fails with the exception HttpClient would throw - see EmuSen_Backend.md §16.
    public sealed unsafe class NativeDownload : IDisposable
    {
        public const int Running = 0, Done = 1, Failed = 2, Cancelled = 3;

        public static bool Active => ServiceLibrary.Active;

        private nint _handle;

        // testRoots is a PEM text trusted beside the system's store, for a test's own server; nothing a player sets reaches it.
        public NativeDownload(string url, TimeSpan timeout, string? testRoots = null)
        {
            byte[] request = ServiceLibrary.Request(w =>
            {
                w.WriteString("url", url);
                if (timeout == Timeout.InfiniteTimeSpan) w.WriteNull("timeout_ms"); else w.WriteNumber("timeout_ms", (long)Math.Ceiling(timeout.TotalMilliseconds));
                ServiceLibrary.WriteText(w, "test_roots", testRoots);
            });
            var start = (delegate* unmanaged[Cdecl]<byte*, nuint, nint>)Export("emusen_net_download_start");
            fixed (byte* r = request) _handle = start(r, (nuint)request.Length);
            if (_handle == 0) throw new ServiceFault(ServiceFault.Parse, ServiceLibrary.LastError());
        }

        // The body of url, as HttpClient.GetByteArrayAsync gives it, or the exception it throws.
        public static byte[] Fetch(string url, TimeSpan timeout, string? testRoots = null)
        {
            using var download = new NativeDownload(url, timeout, testRoots);
            while (download.Wait(TimeSpan.FromSeconds(1)) == Running) { }
            return download.Body();
        }

        // Where it stands, the body's bytes so far, and its length, -1 until known; drained by the caller, never pushed.
        public (int State, long Received, long Total) Progress()
        {
            ulong received;
            long total;
            int state = ((delegate* unmanaged[Cdecl]<nint, ulong*, long*, int>)Export("emusen_net_download_progress"))(Live(), &received, &total);
            return (state, (long)received, total);
        }

        public int Wait(TimeSpan timeout) =>
            ((delegate* unmanaged[Cdecl]<nint, uint, int>)Export("emusen_net_download_wait"))(Live(), (uint)Math.Clamp(timeout.TotalMilliseconds, 0, uint.MaxValue));

        public void Cancel() => ((delegate* unmanaged[Cdecl]<nint, void>)Export("emusen_net_download_cancel"))(Live());

        // The body of a download that is done; the failure's exception for one that is not.
        public byte[] Body()
        {
            var body = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)Export("emusen_net_download_body");
            long length = body(Live(), null, 0);
            if (length < 0) throw Failure();
            var bytes = new byte[length];
            fixed (byte* b = bytes) body(_handle, b, (nuint)bytes.Length);
            return bytes;
        }

        // The exception HttpClient throws for the failure emusen-net reports - see EmuSen_Backend.md §16.2.
        public Exception Failure()
        {
            var failure = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)Export("emusen_net_download_failure");
            long length = failure(Live(), null, 0);
            if (length < 0) return new InvalidOperationException("the download has not ended");
            var bytes = new byte[length];
            fixed (byte* b = bytes) failure(_handle, b, (nuint)bytes.Length);
            using JsonDocument d = JsonDocument.Parse(bytes);
            JsonElement f = d.RootElement;
            string message = f.GetProperty("message").GetString()!;
            string? inner = ServiceLibrary.Text(f, "inner");
            return f.GetProperty("kind").GetString() switch
            {
                "status" => new HttpRequestException(HttpRequestError.Unknown, message, null, (HttpStatusCode)f.GetProperty("status").GetInt32()),
                "timeout" => new TaskCanceledException(message, new TimeoutException(inner)),
                "ended" or "unanswered" => new HttpRequestException(HttpRequestError.ResponseEnded, message, new HttpIOException(HttpRequestError.ResponseEnded, inner)),
                "tls" => new HttpRequestException(HttpRequestError.SecureConnectionError, message, new AuthenticationException(inner)),
                "connect" => new HttpRequestException(HttpRequestError.ConnectionError, message, new SocketException((int)SocketError.SocketError, inner)),
                "resolve" => new HttpRequestException(HttpRequestError.NameResolutionError, message, new SocketException((int)SocketError.HostNotFound, inner)),
                "invalid" => new HttpRequestException(HttpRequestError.InvalidResponse, message),
                "limit" => new HttpRequestException(HttpRequestError.ConfigurationLimitExceeded, message),
                "cancelled" => new OperationCanceledException(message),
                "url" => new InvalidOperationException(message),
                "scheme" => new NotSupportedException(message),
                _ => new HttpRequestException(message, inner is null ? null : new IOException(inner)),
            };
        }

        private nint Live() => _handle != 0 ? _handle : throw new ObjectDisposedException(nameof(NativeDownload));

        private static nint Export(string name)
        {
            nint export = PlatformLibrary.Export(name);
            return export != 0 ? export : throw new InvalidOperationException($"{PlatformLibrary.FileName} has no {name}");
        }

        public void Dispose()
        {
            if (_handle == 0) return;
            ((delegate* unmanaged[Cdecl]<nint, void>)Export("emusen_net_download_free"))(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~NativeDownload() => Dispose();
    }
}
