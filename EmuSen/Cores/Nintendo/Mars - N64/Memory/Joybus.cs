using System;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The PIF walking the command block a game left in PIF RAM, channel by channel - see Mars_Serial.md §3.
    public static class Joybus
    {
        public const byte End = 0xFE;
        public const byte Skip = 0xFF;
        public const byte Pad = 0xFD;

        // A length of zero is a channel with nothing to ask, which is not the same as the info command of the same number - see §3.
        public const byte Silent = 0x00;

        // Set in the length byte when nothing answered, and beside it when the reply was longer than the room for it - see §3.3.
        public const byte NoReply = 0x80;
        public const byte OverRun = 0x40;

        private const byte Info = 0x00, Reset = 0xFF, State = 0x01, PakRead = 0x02, PakWrite = 0x03;

        // The block runs to its end byte or to the last byte of PIF RAM, whichever comes first - see §3.
        public static void Run(byte[] ram, Controller[] ports, SaveChip? cartridge = null)
        {
            int at = 0;
            int channel = 0;

            while (at < ram.Length)
            {
                byte length = ram[at];

                if (length == End) break;

                // A zero says this channel has no command; padding says nothing at all - see §3.
                if (length == Silent)
                {
                    channel++;
                    at++;
                    continue;
                }

                if (length is Pad or Skip)
                {
                    at++;
                    continue;
                }

                if (++at >= ram.Length || ram[at] == End) break;

                int send = length & 0x3F;
                int receive = ram[at] & 0x3F;
                int lengthAt = at;
                int command = ++at;

                if (command + send > ram.Length || command + send + receive > ram.Length) break;

                Answer(ram, ports, cartridge, channel, command, send, receive, lengthAt);

                at = command + send + receive;
                channel++;
            }

            // The PIF clears the byte it was started by, so a block runs once - see §2.
            ram[^1] = 0;
        }

        private static void Answer(byte[] ram, Controller[] ports, SaveChip? cartridge, int channel, int command, int send, int receive, int lengthAt)
        {
            Controller? port = channel < ports.Length ? ports[channel] : null;
            byte id = send > 0 ? ram[command] : Skip;
            int wrote = 0;

            // Every channel past the four ports reaches the cartridge, as the FPGA core routes them - see Mars_Save.md §2.
            bool answered = channel >= ports.Length
                ? cartridge != null && cartridge.AnswerJoybus(ram, command, send, receive, out wrote)
                : port is { Present: true } && Answered(ram, port, id, command, send, receive, out wrote);

            if (!answered)
            {
                ram[lengthAt] = (byte)(NoReply | receive);
                return;
            }

            ram[lengthAt] = (byte)((wrote > receive ? OverRun : 0) | receive);
        }

        // The controller's two commands, and the pak's two when one is in the slot - see §3.2 and Mars_Save.md §5.
        private static bool Answered(byte[] ram, Controller port, byte id, int command, int send, int receive, out int wrote)
        {
            wrote = 0;

            switch (id)
            {
                case Info:
                case Reset:
                    wrote = 3;
                    Reply(ram, command + send, receive, 0x05, 0x00, (byte)(port.Pak != null ? 0x01 : 0x02));
                    return true;

                case State:
                    wrote = 4;
                    Reply(ram, command + send, receive,
                        (byte)(port.Buttons >> 8), (byte)port.Buttons, (byte)port.StickX, (byte)port.StickY);
                    return true;

                case PakRead:
                    if (port.Pak is null || send < 3) return false;
                    wrote = ControllerPak.ChunkSize + 1;
                    Reply(ram, command + send, receive, PakChunk(port.Pak, ram, command));
                    return true;

                case PakWrite:
                    if (port.Pak is null || send < 3 + ControllerPak.ChunkSize) return false;
                    var data = ram.AsSpan(command + 3, ControllerPak.ChunkSize);
                    port.Pak.Write(ControllerPak.Address(ram[command + 1], ram[command + 2]), data);
                    wrote = 1;
                    Reply(ram, command + send, receive, ControllerPak.DataCrc(data));
                    return true;

                default:
                    return false;
            }
        }

        // Thirty-two bytes and their CRC.
        private static byte[] PakChunk(ControllerPak pak, byte[] ram, int command)
        {
            var reply = new byte[ControllerPak.ChunkSize + 1];
            pak.Read(ControllerPak.Address(ram[command + 1], ram[command + 2]), reply.AsSpan(0, ControllerPak.ChunkSize));
            reply[^1] = ControllerPak.DataCrc(reply.AsSpan(0, ControllerPak.ChunkSize));
            return reply;
        }

        // A reply longer than the room for it is cut; a shorter one leaves the rest of the room alone - see §3.3.
        public static void Reply(byte[] ram, int at, int receive, params byte[] bytes)
        {
            for (int i = 0; i < bytes.Length && i < receive; i++) ram[at + i] = bytes[i];
        }
    }
}
