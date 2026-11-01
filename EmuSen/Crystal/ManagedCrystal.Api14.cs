using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace EmuSen.Crystal.Managed
{
    // The functions 1.4 added: the ring and one frame advance - see EmuSen_Backend.md §33.
    public static unsafe partial class ManagedCrystal
    {
        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static nint emusen_crystal_picture_ring(nint rt, uint session)
        {
            try { return SessionOf(rt, session).PictureRing(); }
            catch (Exception e)
            {
                Fail(e);
                return 0;
            }
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        private static int emusen_crystal_frame_advance(nint rt, uint session)
        {
            try { ManagedSession s = SessionOf(rt, session); return Done(s.FrameAdvance); }
            catch (Exception e) { return (int)Fail(e); }
        }
    }
}
