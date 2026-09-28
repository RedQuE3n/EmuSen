using Silk.NET.Vulkan;

namespace EmuSen.Serenity.Slang
{
    // What a bench hears from the shader path, GPU marks and CPU phases; null outside a bench - see EmuSen_Serenity.md §8.1.
    internal interface ISlangProbe
    {
        void Mark(CommandBuffer commands, int point);
        void Phase(int point);
    }

    // The one listener, set by a bench and read with a null check at each point - see EmuSen_Serenity.md §8.1.
    internal static class SlangProbe
    {
        public static ISlangProbe? Current;

        public const int SubmissionBegin = -1, SubmissionEnd = -2;
        public const int Submitted = 1, Waited = 2;
        public const int AdvanceBegin = 10, AdvanceEnd = 11;
        public const int RenderBegin = 20, Bound = 21, Ran = 22, Copied = 23;
        public const int ImageMade = 30;
        public const int FlushBegin = 40, FlushEnd = 41;
    }
}
