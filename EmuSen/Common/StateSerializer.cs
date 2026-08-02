using System;
using System.IO;
using System.Linq;
using System.Reflection;

namespace EmuSen.Common
{
    // Marks a field as NOT part of save-state data. Used for three kinds of
    // fields that reflection would otherwise mishandle:
    //   1. Dispatch tables holding delegates (Spc700._instructions, Ppu's
    //      register table) - delegates aren't meaningfully serializable, and
    //      these get rebuilt identically by BuildOpcodeTable()/
    //      BuildRegisterTable() every time a fresh Spc700/Ppu is
    //      constructed, so they're wiring, not data. (The 65816's own table
    //      is no longer one of these - see Venus_CPU.md §9.)
    //   2. Back-references to an already-owned object (Dma._bus pointing
    //      back to its owning MemoryBus, MemoryBus's own references to the
    //      Cartridge/Spc700 that EmulatorSession already owns and saves
    //      directly) - these are correctly re-wired by the normal
    //      constructors before a load ever happens, and serializing them
    //      would either duplicate data or create a reference cycle.
    //   3. Transient, non-game-state buffers (SDsp.AudioBuffer - pending
    //      output samples, not something a save state needs to restore).
    [AttributeUsage(AttributeTargets.Field)]
    public class SkipInStateAttribute : Attribute { }

    // An alias of an array another field already serializes - written by the
    // pre-v1 format, skipped since. See EmuSen_Save_States.md §2.
    [AttributeUsage(AttributeTargets.Field)]
    public class AliasOfSerializedFieldAttribute : Attribute { }

    // Walks every instance field (public and private, excluding static/const
    // and anything marked [SkipInState]) of an object graph, in a fixed
    // alphabetical order so write and read always agree. Fields of an
    // EXISTING array (byte[], DspVoice[], DmaChannel[], etc.) are filled in
    // place rather than reallocated - every array in this project's state
    // graph is already sized correctly by its owning class's constructor
    // before a save or load ever happens.
    //
    // Still has no field-name tagging: adding, removing, or reordering a
    // field breaks older files unless the version is bumped and a read path
    // kept for the old layout - see EmuSen_Save_States.md §1/§3.
    public static class StateSerializer
    {
        public static void Write(BinaryWriter w, object obj)
        {
            foreach (FieldInfo field in GetStateFields(obj.GetType(), includeAliases: false))
            {
                WriteValue(w, field.FieldType, field.GetValue(obj));
            }
        }

        // includeAliases: true only when reading a pre-v1 file - see EmuSen_Save_States.md §2.
        public static void Read(BinaryReader r, object obj, bool includeAliases = false)
        {
            foreach (FieldInfo field in GetStateFields(obj.GetType(), includeAliases))
            {
                ReadValue(r, field, obj, includeAliases);
            }
        }

        private static FieldInfo[] GetStateFields(Type type, bool includeAliases)
        {
            return type
                .GetFields(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)
                .Where(f => f.GetCustomAttribute<SkipInStateAttribute>() == null)
                .Where(f => includeAliases || f.GetCustomAttribute<AliasOfSerializedFieldAttribute>() == null)
                .OrderBy(f => f.Name, StringComparer.Ordinal)
                .ToArray();
        }

        private static void WriteValue(BinaryWriter w, Type t, object? value)
        {
            if (t == typeof(byte)) { w.Write((byte)value!); return; }
            if (t == typeof(sbyte)) { w.Write((sbyte)value!); return; }
            if (t == typeof(bool)) { w.Write((bool)value!); return; }
            if (t == typeof(short)) { w.Write((short)value!); return; }
            if (t == typeof(ushort)) { w.Write((ushort)value!); return; }
            if (t == typeof(int)) { w.Write((int)value!); return; }
            if (t == typeof(uint)) { w.Write((uint)value!); return; }
            if (t == typeof(long)) { w.Write((long)value!); return; }
            if (t == typeof(ulong)) { w.Write((ulong)value!); return; }
            if (t == typeof(float)) { w.Write((float)value!); return; }
            if (t == typeof(string)) { w.Write((string?)value ?? ""); return; }
            if (t.IsEnum) { w.Write(Convert.ToInt32(value)); return; }

            if (t == typeof(byte[])) { w.Write((byte[])value!); return; }
            if (t == typeof(ushort[])) { foreach (ushort v in (ushort[])value!) w.Write(v); return; }
            if (t == typeof(short[])) { foreach (short v in (short[])value!) w.Write(v); return; }
            if (t == typeof(int[])) { foreach (int v in (int[])value!) w.Write(v); return; }
            if (t == typeof(bool[])) { foreach (bool v in (bool[])value!) w.Write(v); return; }

            if (t.IsArray)
            {
                // Array of a reference type (DspVoice[], DmaChannel[]) -
                // every element is assumed non-null and pre-constructed by
                // the owning class, matching how this project actually
                // initializes these (e.g. Dma's constructor eagerly `new`s
                // all 8 DmaChannel instances up front).
                foreach (object item in (Array)value!) Write(w, item);
                return;
            }

            // An interface-typed field (Cpu._bus, which is a MemoryBus for the
            // S-CPU and an Sa1Bus for the SA-1) is walked exactly like a class:
            // Write() below keys off the runtime type either way.
            if (t.IsClass || t.IsInterface)
            {
                bool hasValue = value != null;
                w.Write(hasValue);
                if (hasValue) Write(w, value!);
                return;
            }

            throw new NotSupportedException($"StateSerializer: unsupported field type {t} - add a case or mark it [SkipInState].");
        }

        private static void ReadValue(BinaryReader r, FieldInfo field, object owner, bool includeAliases)
        {
            Type t = field.FieldType;

            if (t == typeof(byte)) { field.SetValue(owner, r.ReadByte()); return; }
            if (t == typeof(sbyte)) { field.SetValue(owner, r.ReadSByte()); return; }
            if (t == typeof(bool)) { field.SetValue(owner, r.ReadBoolean()); return; }
            if (t == typeof(short)) { field.SetValue(owner, r.ReadInt16()); return; }
            if (t == typeof(ushort)) { field.SetValue(owner, r.ReadUInt16()); return; }
            if (t == typeof(int)) { field.SetValue(owner, r.ReadInt32()); return; }
            if (t == typeof(uint)) { field.SetValue(owner, r.ReadUInt32()); return; }
            if (t == typeof(long)) { field.SetValue(owner, r.ReadInt64()); return; }
            if (t == typeof(ulong)) { field.SetValue(owner, r.ReadUInt64()); return; }
            if (t == typeof(float)) { field.SetValue(owner, r.ReadSingle()); return; }
            if (t == typeof(string)) { field.SetValue(owner, r.ReadString()); return; }
            if (t.IsEnum) { field.SetValue(owner, Enum.ToObject(t, r.ReadInt32())); return; }

            if (t == typeof(byte[]))
            {
                byte[] arr = (byte[])field.GetValue(owner)!;
                int n = r.Read(arr, 0, arr.Length);
                if (n != arr.Length) throw new InvalidDataException($"State file truncated reading {field.Name} (expected {arr.Length} bytes, got {n}).");
                return;
            }
            if (t == typeof(ushort[])) { var arr = (ushort[])field.GetValue(owner)!; for (int i = 0; i < arr.Length; i++) arr[i] = r.ReadUInt16(); return; }
            if (t == typeof(short[])) { var arr = (short[])field.GetValue(owner)!; for (int i = 0; i < arr.Length; i++) arr[i] = r.ReadInt16(); return; }
            if (t == typeof(int[])) { var arr = (int[])field.GetValue(owner)!; for (int i = 0; i < arr.Length; i++) arr[i] = r.ReadInt32(); return; }
            if (t == typeof(bool[])) { var arr = (bool[])field.GetValue(owner)!; for (int i = 0; i < arr.Length; i++) arr[i] = r.ReadBoolean(); return; }

            if (t.IsArray)
            {
                foreach (object item in (Array)field.GetValue(owner)!) Read(r, item, includeAliases);
                return;
            }

            if (t.IsClass || t.IsInterface)
            {
                bool hasValue = r.ReadBoolean();
                if (hasValue) Read(r, field.GetValue(owner)!, includeAliases);
                return;
            }

            throw new NotSupportedException($"StateSerializer: unsupported field type {t} - add a case or mark it [SkipInState].");
        }
    }
}
