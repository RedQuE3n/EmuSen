namespace EmuSen.WiseMan.Serenity
{
    // Whether the process has the Vulkan device the frame control's filters are drawn by; a case that draws one passes by without it - see EmuSen_DevicePicture.md §21.
    internal static class ChainDevice
    {
        public static bool Missing => EmuSen.Serenity.Slang.SlangRunner.SharedDevice is null;
    }
}
