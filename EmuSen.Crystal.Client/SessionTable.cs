using System.Text;
using EmuSen.DianaSh.DianaSh.Bin.Native;

namespace EmuSen.Crystal.Client
{
    // A session service table's calls, each failure in the table's own words: the debugger's one surface since frontend API 2.0 - see EmuSen_RustPlatform.md §47.23.
    internal sealed unsafe class SessionTable(nint table)
    {
        public EmuSenSessionV1* T { get; } = (EmuSenSessionV1*)table;

        public CrystalException Failure(long status) =>
            new((int)status, Encoding.UTF8.GetString(CrystalRuntime.Bytes((o, n) => T->last_error(T->ctx, o, n))));

        public long Check(long result) => result < 0 ? throw Failure(result) : result;

        public byte[] Bytes(CrystalCopyOut call) => CrystalRuntime.Bytes(call, Check);

        public string Text(CrystalCopyOut call) => Encoding.UTF8.GetString(Bytes(call));
    }
}
