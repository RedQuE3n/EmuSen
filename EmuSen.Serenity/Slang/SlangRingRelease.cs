using System;
using EmuSen.Serenity.Shaders;

namespace EmuSen.Serenity.Slang
{
    // Gives a ring's slots back while the screen is not drawing on the device, so a hosted core never waits on a picture nobody takes - see EmuSen_Backend.md §33.2.
    public sealed class SlangRingRelease : IDisposable
    {
        private SlangChain? _chain;
        private SlangVulkan? _device;

        public void Release(RingPicture picture, int width, int height)
        {
            if (!ReferenceEquals(_device, picture.Ring.Owner))
            {
                _chain?.Dispose();
                _chain = new SlangChain(picture.Ring.Owner, SlangPreset.Load(BuiltInPreset.Write(BuiltInShaders.AsFilter(ShaderEffect.None, false), null)));
                _device = picture.Ring.Owner;
            }
            _chain!.AdvanceRing(picture.Ring, picture.Slot, picture.Value, width, height);
            _chain.Render(1, 1);
        }

        public void Dispose()
        {
            _chain?.Dispose();
            (_chain, _device) = (null, null);
        }
    }
}
