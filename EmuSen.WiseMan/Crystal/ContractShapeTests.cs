using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using EmuSen.Crystal.Client.Contract;

namespace EmuSen.WiseMan.Crystal
{
    // The contract layer names only what C can: integers, doubles, pointers to them and to its own size-prefixed structs, and function pointers of such - see EmuSen_CoreAPI.md §9.3 and EmuSen_Backend.md §14.
    public class ContractShapeTests
    {
        private static readonly Assembly Binding = typeof(CrystalExports).Assembly;

        private static IEnumerable<Type> ContractTypes() =>
            Binding.GetExportedTypes().Where(t => t.Namespace == "EmuSen.Crystal.Client.Contract");

        private static readonly HashSet<Type> Scalars = new()
        {
            typeof(void), typeof(byte), typeof(sbyte), typeof(short), typeof(ushort), typeof(int), typeof(uint), typeof(long), typeof(ulong),
            typeof(nint), typeof(nuint), typeof(double),
        };

        // Why a type is not C-expressible here, or null.
        private static string? Refusal(Type type)
        {
            if (type.IsFunctionPointer)
                return new[] { type.GetFunctionPointerReturnType() }.Concat(type.GetFunctionPointerParameterTypes()).Select(t => Refusal(t.UnderlyingSystemType)).FirstOrDefault(r => r is not null)
                    ?? (type.IsUnmanagedFunctionPointer ? null : "a managed function pointer");
            if (type.IsPointer) return Refusal(type.GetElementType()!);
            if (Scalars.Contains(type)) return null;
            if (type.IsValueType && !type.IsEnum && !type.IsGenericType && type.Namespace == "EmuSen.Crystal.Client.Contract")
                return type.GetFields(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic).Select(f => Refusal(f.GetModifiedFieldType().UnderlyingSystemType is { IsFunctionPointer: true } fp ? f.GetModifiedFieldType() : f.FieldType)).FirstOrDefault(r => r is not null);
            return $"{type} is not a C type";
        }

        [Fact]
        public void Every_public_type_of_the_contract_is_a_static_class_or_a_plain_struct()
        {
            foreach (Type t in ContractTypes())
            {
                bool staticClass = t.IsClass && t.IsAbstract && t.IsSealed;
                Assert.True(staticClass || Refusal(t) is null, $"{t.Name}: {(t.IsClass ? "a class with instances" : Refusal(t))}");
                Assert.False(t.IsGenericType, t.Name);
            }
        }

        [Fact]
        public void Every_function_constant_and_field_of_the_contract_is_C_expressible()
        {
            var problems = new List<string>();
            foreach (Type t in ContractTypes())
            {
                foreach (MethodInfo m in t.GetMethods(BindingFlags.Public | BindingFlags.Static | BindingFlags.DeclaredOnly))
                {
                    if (m.IsSpecialName) problems.Add($"{t.Name}.{m.Name}: a property or operator");
                    if (m.IsGenericMethod) problems.Add($"{t.Name}.{m.Name}: generic");
                    foreach (Type p in m.GetParameters().Select(p => p.ParameterType).Append(m.ReturnType))
                        if (Refusal(p) is { } why) problems.Add($"{t.Name}.{m.Name}: {why}");
                }
                foreach (FieldInfo f in t.GetFields(BindingFlags.Public | BindingFlags.Static | BindingFlags.Instance | BindingFlags.DeclaredOnly))
                {
                    Type type = !f.IsLiteral && f.GetModifiedFieldType() is { IsFunctionPointer: true } pointer ? pointer : f.FieldType;
                    if (f.IsStatic && !f.IsLiteral) problems.Add($"{t.Name}.{f.Name}: a public static that is not a constant");
                    if (Refusal(type) is { } why) problems.Add($"{t.Name}.{f.Name}: {why}");
                }
                if (t.GetProperties(BindingFlags.Public | BindingFlags.Static | BindingFlags.Instance).Any()) problems.Add($"{t.Name}: public properties");
                if (t.GetEvents(BindingFlags.Public | BindingFlags.Static | BindingFlags.Instance).Any()) problems.Add($"{t.Name}: events");
            }
            Assert.Empty(problems);
        }

        // The check is held to seeing what it exists to refuse.
        [Fact]
        public void The_check_refuses_what_C_cannot_express()
        {
            Assert.NotNull(Refusal(typeof(string)));
            Assert.NotNull(Refusal(typeof(byte[])));
            Assert.NotNull(Refusal(typeof(Action)));
            Assert.NotNull(Refusal(typeof(List<int>)));
            Assert.NotNull(Refusal(typeof(Span<byte>)));
            Assert.NotNull(Refusal(typeof(bool)));
            Assert.Null(Refusal(typeof(CrystalFrameInfo*)));
            Assert.Null(Refusal(typeof(long)));
        }
    }
}
