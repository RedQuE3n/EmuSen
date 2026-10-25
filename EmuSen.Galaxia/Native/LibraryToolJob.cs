using System;
using System.Text;

namespace EmuSen.Galaxia.Native
{
    // A job of the library tool as a frontend follows it: events taken by polling, a stop asked, the job freed.
    public interface IToolJob : IDisposable
    {
        // The next event's JSON line, waiting up to waitMs for one; null when none came in time or the job has ended.
        string? Next(int waitMs);

        // True once the last event was taken.
        bool Ended { get; }

        void Stop();
    }

    // The library tool's four calls, emusen_tool_job_start/next/stop/free; the library never calls its host, so the host polls - see EmuSen_GamePackages.md §12.3 and §13.
    public sealed unsafe class LibraryToolJob : IToolJob
    {
        private const int Absent = -1030;
        private const int Parse = -1025;

        private static readonly Lazy<(nint Start, nint Next, nint Stop, nint Free, nint LastError)> Exports = new(() =>
            (PlatformLibrary.Export("emusen_tool_job_start"), PlatformLibrary.Export("emusen_tool_job_next"), PlatformLibrary.Export("emusen_tool_job_stop"),
             PlatformLibrary.Export("emusen_tool_job_free"), PlatformLibrary.Export("emusen_platform_last_error")));

        private nint _job;
        private byte[] _buffer = new byte[4096];

        private LibraryToolJob(nint job) => _job = job;

        // Whether the tool can run here: the platform library is loaded and has the four calls.
        public static bool Available
        {
            get
            {
                var e = Exports.Value;
                return e.Start != 0 && e.Next != 0 && e.Stop != 0 && e.Free != 0 && e.LastError != 0;
            }
        }

        // Why the platform library is or is not in use, for the words of a frontend that cannot run the tool.
        public static string Report => PlatformLibrary.Report;

        // A job started from a request; a request that is not one is a FormatException with the tool's words.
        public static IToolJob Start(string request)
        {
            if (!Available) throw new InvalidOperationException($"{PlatformLibrary.FileName} is not in use ({PlatformLibrary.Report})");
            byte[] text = Encoding.UTF8.GetBytes(request);
            nint job = 0;
            int status;
            fixed (byte* p = text) status = ((delegate* unmanaged[Cdecl]<byte*, nuint, nint*, int>)Exports.Value.Start)(p, (nuint)text.Length, &job);
            if (status == Parse) throw new FormatException(LastError());
            if (status < 0 || job == 0) throw new InvalidOperationException($"{PlatformLibrary.FileName}: the job did not start ({status}): {LastError()}");
            return new LibraryToolJob(job);
        }

        public bool Ended { get; private set; }

        public string? Next(int waitMs)
        {
            if (_job == 0 || Ended) return null;
            var next = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long>)Exports.Value.Next;
            while (true)
            {
                long length;
                fixed (byte* b = _buffer) length = next(_job, (uint)Math.Max(0, waitMs), b, (nuint)_buffer.Length);
                if (length == 0) { Ended = true; return null; }
                if (length == Absent) return null;
                if (length < 0) throw new InvalidOperationException($"{PlatformLibrary.FileName}: the job's next event failed ({length}): {LastError()}");
                if (length <= _buffer.Length) return Encoding.UTF8.GetString(_buffer, 0, (int)length);
                _buffer = new byte[length];
                waitMs = 0;
            }
        }

        public void Stop()
        {
            if (_job != 0) ((delegate* unmanaged[Cdecl]<nint, int>)Exports.Value.Stop)(_job);
        }

        // Stops the job, waits for it to end, and frees it.
        public void Dispose()
        {
            if (_job == 0) return;
            ((delegate* unmanaged[Cdecl]<nint, int>)Exports.Value.Free)(_job);
            _job = 0;
        }

        private static string LastError()
        {
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)Exports.Value.LastError;
            byte[] buffer = new byte[1024];
            long length;
            fixed (byte* b = buffer) length = call(b, (nuint)buffer.Length);
            if (length <= 0) return "";
            if (length > buffer.Length)
            {
                buffer = new byte[length];
                fixed (byte* b = buffer) length = call(b, (nuint)buffer.Length);
            }
            return Encoding.UTF8.GetString(buffer, 0, (int)Math.Min(length, buffer.Length));
        }
    }
}
