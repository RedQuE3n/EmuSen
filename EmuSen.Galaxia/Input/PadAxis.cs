namespace EmuSen.Galaxia.Input
{
    // The generic pad's analog axes: sticks run -1 to 1 with right and down positive, as the RetroPad's do, triggers 0 to 1 - see EmuSen_Input.md §7.
    public enum PadAxis
    {
        LeftX,
        LeftY,
        RightX,
        RightY,
        LeftTrigger,
        RightTrigger,
    }
}
