namespace EmuSen.Mistress
{
    // Which way a frame's picture reached the screen, named in the log each time it changes - see EmuSen_Backend.md §14.13.
    public enum PicturePath { InRing, OnDevice, RingReadBack, DeviceReadBack, NoDeviceOffered, EngineTookNone }

    public static class PicturePaths
    {
        // The path a frame takes from what the runtime says of its picture and whether the screen presents on the device.
        public static PicturePath Of(bool inRing, bool onDevice, bool presentsOnDevice, bool deviceOffered) =>
            inRing ? (presentsOnDevice ? PicturePath.InRing : PicturePath.RingReadBack)
            : onDevice ? (presentsOnDevice ? PicturePath.OnDevice : PicturePath.DeviceReadBack)
            : deviceOffered ? PicturePath.EngineTookNone : PicturePath.NoDeviceOffered;

        public static string Words(PicturePath path) => path switch
        {
            PicturePath.InRing => "in a ring: the hosted core draws into a ring made on the presenter's device and the chain reads it there",
            PicturePath.OnDevice => "on the device: the core draws on the presenter's device and the chain reads it there",
            PicturePath.RingReadBack => "read back: the core drew into the ring, but the screen is not presenting on the device yet or any more",
            PicturePath.DeviceReadBack => "read back: the core drew on the lent device, but the screen is not presenting on the device yet or any more",
            PicturePath.NoDeviceOffered => "read back: the picture is not presented on the device, so no device was lent",
            _ => "read back: the engine took neither the lent device nor a ring",
        };
    }
}
