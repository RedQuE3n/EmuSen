using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Arithmetic, and the four instructions that trap instead of wrapping - see Mars_Cpu.md §2.1.
    public sealed partial class Cpu
    {
        // The whole register is shifted and the result truncated afterwards, not the other way round - see §14.
        private void ShiftRightArithmetic(uint instruction, int amount) =>
            Write32(Rd(instruction), (uint)((long)Read(Rt(instruction)) >> amount));

        private void AddImmediate(uint instruction, bool trapOnOverflow)
        {
            int left = (int)(uint)Read(Rs(instruction));
            int right = (int)SignedImmediate(instruction);
            int result = unchecked(left + right);

            if (trapOnOverflow && OverflowedAdd(left, right, result)) throw Raise(ExceptionCode.Overflow, CurrentPc);

            Write32(Rt(instruction), (uint)result);
        }

        private void AddImmediate64(uint instruction, bool trapOnOverflow)
        {
            long left = (long)Read(Rs(instruction));
            long right = SignedImmediate(instruction);
            long result = unchecked(left + right);

            if (trapOnOverflow && OverflowedAdd(left, right, result)) throw Raise(ExceptionCode.Overflow, CurrentPc);

            Write(Rt(instruction), (ulong)result);
        }

        private void Add(uint instruction, bool trapOnOverflow)
        {
            int left = (int)(uint)Read(Rs(instruction));
            int right = (int)(uint)Read(Rt(instruction));
            int result = unchecked(left + right);

            if (trapOnOverflow && OverflowedAdd(left, right, result)) throw Raise(ExceptionCode.Overflow, CurrentPc);

            Write32(Rd(instruction), (uint)result);
        }

        private void Subtract(uint instruction, bool trapOnOverflow)
        {
            int left = (int)(uint)Read(Rs(instruction));
            int right = (int)(uint)Read(Rt(instruction));
            int result = unchecked(left - right);

            if (trapOnOverflow && OverflowedSubtract(left, right, result)) throw Raise(ExceptionCode.Overflow, CurrentPc);

            Write32(Rd(instruction), (uint)result);
        }

        private void Add64(uint instruction, bool trapOnOverflow)
        {
            long left = (long)Read(Rs(instruction));
            long right = (long)Read(Rt(instruction));
            long result = unchecked(left + right);

            if (trapOnOverflow && OverflowedAdd(left, right, result)) throw Raise(ExceptionCode.Overflow, CurrentPc);

            Write(Rd(instruction), (ulong)result);
        }

        private void Subtract64(uint instruction, bool trapOnOverflow)
        {
            long left = (long)Read(Rs(instruction));
            long right = (long)Read(Rt(instruction));
            long result = unchecked(left - right);

            if (trapOnOverflow && OverflowedSubtract(left, right, result)) throw Raise(ExceptionCode.Overflow, CurrentPc);

            Write(Rd(instruction), (ulong)result);
        }

        private void SetLessThanImmediate(uint instruction, bool unsigned)
        {
            ulong result = unsigned
                ? Read(Rs(instruction)) < (ulong)SignedImmediate(instruction) ? 1UL : 0UL
                : (long)Read(Rs(instruction)) < SignedImmediate(instruction) ? 1UL : 0UL;

            Write(Rt(instruction), result);
        }

        // The sign rule rather than a widened compare, so the 32- and 64-bit forms read alike.
        private static bool OverflowedAdd(long left, long right, long result) =>
            ((left ^ result) & (right ^ result)) < 0;

        private static bool OverflowedSubtract(long left, long right, long result) =>
            ((left ^ right) & (left ^ result)) < 0;
    }
}
