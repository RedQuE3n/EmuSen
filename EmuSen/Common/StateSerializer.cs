using System;
using System.IO;
using System.Linq;
using System.Reflection;

namespace EmuSen.Common
{
    // Marks a field as NOT part of save-state data - see Venus_CPU.md §9.
    [AttributeUsage(AttributeTargets.Field)]
    public class SkipInStateAttribute : Attribute { }

    // An array alias another field already serializes, written by the pre-v1 format - see EmuSen_Save_States.md §2.
    [AttributeUsage(AttributeTargets.Field)]
    public class AliasOfSerializedFieldAttribute : Attribute { }

    // A field an older state version wrote and the current one does not: walked only for that older version - see EmuSen_Save_States.md §7.
    [AttributeUsage(AttributeTargets.Field)]
    public class RetiredFromStateAttribute : Attribute { }

    // Walks every instance field (public and private, excluding static/const and anything marked - see EmuSen_Save_States.md §1.
    public static class StateSerializer
    {
        // includeRetired writes an older version's bytes, retired fields included - see EmuSen_Save_States.md §7.
        public static void Write(BinaryWriter w, object obj, bool includeRetired = false)
        {
            foreach (FieldInfo field in GetStateFields(obj.GetType(), includeAliases: false, includeRetired))
            {
                WriteValue(w, field.FieldType, field.GetValue(obj), includeRetired);
            }
        }

        // includeAliases: true only when reading a pre-v1 file - see EmuSen_Save_States.md §2; includeRetired, §7.
        public static void Read(BinaryReader r, object obj, bool includeAliases = false, bool includeRetired = false)
        {
            foreach (FieldInfo field in GetStateFields(obj.GetType(), includeAliases, includeRetired))
            {
                ReadValue(r, field, obj, includeAliases, includeRetired);
            }
        }

        private static FieldInfo[] GetStateFields(Type type, bool includeAliases, bool includeRetired)
        {
            return type
                .GetFields(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)
                .Where(f => f.GetCustomAttribute<SkipInStateAttribute>() == null)
                .Where(f => includeAliases || f.GetCustomAttribute<AliasOfSerializedFieldAttribute>() == null)
                .Where(f => includeRetired || f.GetCustomAttribute<RetiredFromStateAttribute>() == null)
                .OrderBy(f => f.Name, StringComparer.Ordinal)
                .ToArray();
        }

        // A retired scalar or string is consumed and its value dropped; a retired reference is read into what it references - see EmuSen_Save_States.md §7.
        private static void DiscardValue(BinaryReader r, Type t)
        {
            if (t == typeof(string)) { r.ReadString(); return; }
            if (t.IsEnum) { r.ReadInt32(); return; }
            if (t == typeof(bool) || t == typeof(byte) || t == typeof(sbyte)) { r.ReadByte(); return; }
            if (t == typeof(short) || t == typeof(ushort) || t == typeof(char)) { r.ReadUInt16(); return; }
            if (t == typeof(int) || t == typeof(uint) || t == typeof(float)) { r.ReadUInt32(); return; }
            if (t == typeof(long) || t == typeof(ulong)) { r.ReadUInt64(); return; }
            throw new NotSupportedException($"StateSerializer: a retired {t} field cannot be discarded - add a case.");
        }

        private static void WriteValue(BinaryWriter w, Type t, object? value, bool includeRetired)
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
            if (t == typeof(char)) { w.Write((ushort)(char)value!); return; }
            if (t.IsEnum) { w.Write(Convert.ToInt32(value)); return; }

            if (t == typeof(byte[])) { w.Write((byte[])value!); return; }
            if (t == typeof(ushort[])) { foreach (ushort v in (ushort[])value!) w.Write(v); return; }
            if (t == typeof(short[])) { foreach (short v in (short[])value!) w.Write(v); return; }
            if (t == typeof(int[])) { foreach (int v in (int[])value!) w.Write(v); return; }
            if (t == typeof(bool[])) { foreach (bool v in (bool[])value!) w.Write(v); return; }

            // The same bytes the element walk below always wrote for these, so no older state changes - see EmuSen_Save_States.md §5.
            if (t == typeof(uint[])) { foreach (uint v in (uint[])value!) w.Write(v); return; }
            if (t == typeof(long[])) { foreach (long v in (long[])value!) w.Write(v); return; }
            if (t == typeof(ulong[])) { foreach (ulong v in (ulong[])value!) w.Write(v); return; }

            if (t.IsArray)
            {
                // Every element is assumed pre-constructed, which is how the owning classes initialise these.
                foreach (object item in (Array)value!) Write(w, item, includeRetired);
                return;
            }

            // A struct is its fields, in the same order a class's are.
            if (t.IsValueType)
            {
                Write(w, value!, includeRetired);
                return;
            }

            // An interface-typed field (Cpu._bus, which is a MemoryBus for the S-CPU and an Sa1Bus for the SA-1).
            if (t.IsClass || t.IsInterface)
            {
                bool hasValue = value != null;
                w.Write(hasValue);
                if (hasValue) Write(w, value!, includeRetired);
                return;
            }

            throw new NotSupportedException($"StateSerializer: unsupported field type {t} - add a case or mark it [SkipInState].");
        }

        private static void ReadValue(BinaryReader r, FieldInfo field, object owner, bool includeAliases, bool includeRetired)
        {
            Type t = field.FieldType;

            if (includeRetired && (t.IsValueType || t == typeof(string)) && field.GetCustomAttribute<RetiredFromStateAttribute>() != null)
            {
                DiscardValue(r, t);
                return;
            }

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
            if (t == typeof(char)) { field.SetValue(owner, (char)r.ReadUInt16()); return; }
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
            if (t == typeof(uint[])) { var arr = (uint[])field.GetValue(owner)!; for (int i = 0; i < arr.Length; i++) arr[i] = r.ReadUInt32(); return; }
            if (t == typeof(long[])) { var arr = (long[])field.GetValue(owner)!; for (int i = 0; i < arr.Length; i++) arr[i] = r.ReadInt64(); return; }
            if (t == typeof(ulong[])) { var arr = (ulong[])field.GetValue(owner)!; for (int i = 0; i < arr.Length; i++) arr[i] = r.ReadUInt64(); return; }

            if (t.IsArray)
            {
                var array = (Array)field.GetValue(owner)!;
                bool values = t.GetElementType()!.IsValueType;

                // A struct element is read into a box, which has to be stored back or the read is lost - see EmuSen_Save_States.md §5.
                for (int i = 0; i < array.Length; i++)
                {
                    object item = array.GetValue(i)!;
                    Read(r, item, includeAliases, includeRetired);
                    if (values) array.SetValue(item, i);
                }
                return;
            }

            if (t.IsValueType)
            {
                object boxed = field.GetValue(owner)!;
                Read(r, boxed, includeAliases, includeRetired);
                field.SetValue(owner, boxed);
                return;
            }

            if (t.IsClass || t.IsInterface)
            {
                bool hasValue = r.ReadBoolean();
                if (hasValue) Read(r, field.GetValue(owner)!, includeAliases, includeRetired);
                return;
            }

            throw new NotSupportedException($"StateSerializer: unsupported field type {t} - add a case or mark it [SkipInState].");
        }
    }
}
