using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Reflection.Emit;

namespace EmuSen.WiseMan.Fixtures
{
    // A test that reaches Avalonia off the headless session's dispatcher, and the call chain that shows it.
    public sealed record OffDispatcherFinding(string Test, string Chain, string Site);

    // Reads each test's IL for Avalonia work done outside a dispatch - see EmuSen_Settings_Reference.md §4.78.5.
    public static class OffDispatcherScan
    {
        private static readonly Dictionary<short, OpCode> Codes = typeof(OpCodes)
            .GetFields(BindingFlags.Public | BindingFlags.Static)
            .Select(f => (OpCode)f.GetValue(null)!)
            .ToDictionary(c => c.Value);

        private const BindingFlags Declared = BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static | BindingFlags.DeclaredOnly;

        public static IReadOnlyList<OffDispatcherFinding> Scan(Assembly tests, Type avaloniaObject, Func<Assembly, bool> product)
        {
            var scan = new Scanner(tests, avaloniaObject, product);
            var findings = new List<OffDispatcherFinding>();
            foreach (Type type in tests.GetTypes().OrderBy(t => t.FullName, StringComparer.Ordinal))
            {
                MethodInfo[] cases = type.GetMethods(Declared).Where(IsTest).ToArray();
                if (cases.Length == 0) continue;
                foreach (MethodBase method in cases.Concat(Around(type)).OrderBy(m => m.Name, StringComparer.Ordinal))
                {
                    if (scan.Reach(method) is { } hit) findings.Add(new OffDispatcherFinding($"{type.FullName}.{method.Name}", hit.Chain, hit.Site));
                }
            }

            return findings;
        }

        // One method's first site off the dispatcher, or null, for the guard's own controls.
        public static string? Explain(MethodBase method, Assembly tests, Type avaloniaObject, Func<Assembly, bool> product) =>
            new Scanner(tests, avaloniaObject, product).Reach(method) is { } hit ? $"{hit.Site} via {hit.Chain}" : null;

        // What xUnit runs around a case on the test's own thread: the class's constructors and disposal, and its class fixtures'.
        private static IEnumerable<MethodBase> Around(Type type)
        {
            IEnumerable<Type> fixtures = type.GetInterfaces()
                .Where(i => i.IsGenericType && i.GetGenericTypeDefinition().FullName == "Xunit.IClassFixture`1")
                .Select(i => i.GetGenericArguments()[0]);
            foreach (Type owner in fixtures.Prepend(type))
            {
                foreach (ConstructorInfo ctor in owner.GetConstructors(Declared)) yield return ctor;
                foreach (MethodInfo m in owner.GetMethods(Declared).Where(m => m.Name is "Dispose" or "DisposeAsync" or "InitializeAsync")) yield return m;
            }
        }

        private static bool IsTest(MethodInfo method) =>
            method.GetCustomAttributesData().Any(a => Derives(a.AttributeType, "Xunit.FactAttribute"));

        private static bool Derives(Type? type, string name)
        {
            for (; type is not null; type = type.BaseType) if (type.FullName == name) return true;
            return false;
        }

        private static string Name(MethodBase m) => $"{m.DeclaringType?.Name}.{m.Name}";

        private sealed class Scanner(Assembly tests, Type avaloniaObject, Func<Assembly, bool> product)
        {
            private readonly Dictionary<MethodBase, (string Chain, string Site)?> _reach = new();
            private readonly Dictionary<MethodBase, bool> _dispatches = new();

            public (string Chain, string Site)? Reach(MethodBase method)
            {
                if (_reach.TryGetValue(method, out var known)) return known;
                _reach[method] = null;
                var found = Walk(method);
                _reach[method] = found;
                return found;
            }

            private (string Chain, string Site)? Via(MethodBase from, MethodBase to) =>
                Reach(to) is { } hit ? ($"{Name(from)} > {hit.Chain}", hit.Site) : null;

            private bool Follows(MethodBase m) => m.Module.Assembly == tests || product(m.Module.Assembly);

            private (string Chain, string Site)? Walk(MethodBase method)
            {
                foreach (var data in method.GetCustomAttributesData())
                {
                    if (data.AttributeType.FullName is "System.Runtime.CompilerServices.AsyncStateMachineAttribute" or "System.Runtime.CompilerServices.IteratorStateMachineAttribute"
                        && data.ConstructorArguments[0].Value is Type machine
                        && machine.GetMethod("MoveNext", Declared) is { } moveNext
                        && Via(method, moveNext) is { } inMachine) return inMachine;
                }

                MethodBase? target = null;
                var pending = new List<MethodBase>();
                foreach (var (op, operand) in Instructions(method))
                {
                    if (op == OpCodes.Ldftn || op == OpCodes.Ldvirtftn)
                    {
                        target = operand as MethodBase;
                        continue;
                    }

                    if (operand is not MethodBase callee) continue;
                    bool isNew = op == OpCodes.Newobj;
                    if (op != OpCodes.Call && op != OpCodes.Callvirt && !isNew) continue;

                    Type? declaring = callee.DeclaringType;
                    if (isNew && declaring is not null && declaring.IsSubclassOf(typeof(MulticastDelegate)))
                    {
                        if (target is not null) pending.Add(target);
                        target = null;
                        continue;
                    }

                    if (isNew && declaring is not null && Site(declaring) is { } site) return (Name(method), site);
                    if (method.Module.Assembly == tests && callee.Name == "get_UIThread" && declaring?.FullName == "Avalonia.Threading.Dispatcher")
                        return (Name(method), "Dispatcher.UIThread");

                    if (pending.Count > 0 && TakesDelegate(callee))
                    {
                        bool dispatched = Dispatches(callee);
                        foreach (MethodBase handed in pending) if (!dispatched && Via(method, handed) is { } passed) return passed;
                        pending.Clear();
                    }

                    if (Follows(callee) && !Dispatches(callee) && Via(method, callee) is { } deeper) return deeper;
                }

                foreach (MethodBase left in pending) if (Via(method, left) is { } stored) return stored;
                return null;
            }

            private string? Site(Type type)
            {
                if (type.IsAssignableTo(avaloniaObject)) return $"new {type.FullName}";
                return type.Namespace == "Avalonia.Media.Imaging" || type.FullName is "Avalonia.Media.FormattedText" or "Avalonia.Media.TextFormatting.TextLayout"
                    ? $"new {type.FullName}"
                    : null;
            }

            private static bool TakesDelegate(MethodBase m) => m.GetParameters().Any(p => p.ParameterType.IsSubclassOf(typeof(Delegate)));

            // The session's own Dispatch, and any test-assembly method that hands a delegate parameter of its own to one.
            private bool Dispatches(MethodBase m)
            {
                if (m.DeclaringType?.FullName == "Avalonia.Headless.HeadlessUnitTestSession" && m.Name == "Dispatch") return true;
                if (m.Module.Assembly != tests || !TakesDelegate(m)) return false;
                if (_dispatches.TryGetValue(m, out bool known)) return known;
                _dispatches[m] = false;
                bool result = Instructions(m).Any(i => i.Operand is MethodBase c && (i.Op == OpCodes.Call || i.Op == OpCodes.Callvirt) && Dispatches(c));
                _dispatches[m] = result;
                return result;
            }

            private static IEnumerable<(OpCode Op, object? Operand)> Instructions(MethodBase method)
            {
                byte[]? il;
                try { il = method.GetMethodBody()?.GetILAsByteArray(); }
                catch (Exception e) when (e is InvalidOperationException or NotSupportedException or BadImageFormatException) { il = null; }
                if (il is null) yield break;

                Type[]? typeArgs = method.DeclaringType is { IsGenericType: true } t ? t.GetGenericArguments() : null;
                Type[]? methodArgs = method is MethodInfo { IsGenericMethod: true } g ? g.GetGenericArguments() : null;
                for (int i = 0; i < il.Length;)
                {
                    short value = il[i++];
                    if (value == 0xFE) value = unchecked((short)(0xFE00 | il[i++]));
                    if (!Codes.TryGetValue(value, out OpCode op)) yield break;

                    object? operand = null;
                    switch (op.OperandType)
                    {
                        case OperandType.InlineNone: break;
                        case OperandType.ShortInlineBrTarget or OperandType.ShortInlineI or OperandType.ShortInlineVar: i += 1; break;
                        case OperandType.InlineVar: i += 2; break;
                        case OperandType.InlineI8 or OperandType.InlineR: i += 8; break;
                        case OperandType.InlineSwitch: i += 4 + 4 * BitConverter.ToInt32(il, i); break;
                        case OperandType.InlineMethod:
                            int token = BitConverter.ToInt32(il, i);
                            i += 4;
                            try { operand = method.Module.ResolveMethod(token, typeArgs, methodArgs); }
                            catch (Exception e) when (e is ArgumentException or TypeLoadException or System.IO.FileNotFoundException or BadImageFormatException or MissingMethodException) { }
                            break;
                        default: i += 4; break;
                    }

                    yield return (op, operand);
                }
            }
        }
    }
}
