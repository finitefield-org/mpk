using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using Microsoft.Win32.SafeHandles;

namespace Mpk.CSharp2Vir;

internal enum CapturedInputKind
{
    Source,
    Contract,
}

internal sealed class CapturedFile
{
    private readonly byte[] bytes;

    internal CapturedFile(CapturedInputKind kind, string normalizedPath, byte[] bytes)
    {
        Kind = kind;
        NormalizedPath = normalizedPath;
        this.bytes = bytes;
        Sha256 = Convert.ToHexString(SHA256.HashData(bytes)).ToLowerInvariant();
    }

    internal CapturedInputKind Kind { get; }

    internal string NormalizedPath { get; }

    internal int SizeBytes => bytes.Length;

    internal string Sha256 { get; }

    internal ReadOnlySpan<byte> Bytes => bytes;
}

internal sealed class CapturedSnapshot
{
    private readonly CapturedFile[] files;

    internal CapturedSnapshot(Selection selection, CapturedFile[] files)
    {
        Selection = selection;
        this.files = files;
    }

    internal Selection Selection { get; }

    internal int Count => files.Length;

    internal CapturedFile FileAt(int index) => files[index];

    internal CapturedFile Find(CapturedInputKind kind, string normalizedPath)
    {
        CapturedFile? result = null;
        foreach (CapturedFile file in files)
        {
            if (file.Kind == kind && string.Equals(file.NormalizedPath, normalizedPath, StringComparison.Ordinal))
            {
                if (result is not null)
                {
                    throw FrontendFailure.Internal("capture");
                }

                result = file;
            }
        }

        return result ?? throw FrontendFailure.Internal("capture");
    }
}

internal static class SnapshotCapture
{
    private const int SourceFileBytesMaximum = 1_048_576;
    private const int SourceTotalBytesMaximum = 16_777_216;
    private const int ContractFileBytesMaximum = 1_048_576;
    private const int ContractTotalBytesMaximum = 8_388_608;
    private const int SnapshotEntriesMaximum = 512;
    private const int SnapshotTotalBytesMaximum = 33_554_432;

    internal static CapturedSnapshot Capture(string rootPath, Selection selection)
    {
        try
        {
            return CaptureChecked(rootPath, selection);
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (error is IOException || error is UnauthorizedAccessException)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
        }
        catch (Exception error) when (
            error is ArgumentException || error is NotSupportedException || error is OverflowException)
        {
            throw FrontendFailure.Internal("capture");
        }
    }

    private static CapturedSnapshot CaptureChecked(string rootPath, Selection selection)
    {
        if (!OperatingSystem.IsLinux()
            || RuntimeInformation.ProcessArchitecture != Architecture.X64
            || string.IsNullOrEmpty(rootPath)
            || !Path.IsPathFullyQualified(rootPath))
        {
            throw FrontendFailure.Internal("capture");
        }

        string root = Path.GetFullPath(rootPath);
        using SafeFileHandle rootHandle = OpenAbsoluteDirectoryNoFollow(root);
        UnixStat rootStat = FStatChecked(rootHandle);
        if (!IsDirectory(rootStat.Mode))
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
        }

        Dictionary<string, ExpectedNodeKind> expected = BuildExpectedInventory(selection.Raw);
        Dictionary<string, InventoryNode> inventory = InspectInventory(rootHandle, expected);
        ValidateProjectedLimits(selection.Raw, inventory);
        var captured = new List<CapturedFile>(selection.Raw.Sources.Count + selection.Raw.Contracts.Count);
        ulong sourceTotal = 0;
        ulong contractTotal = 0;
        ulong snapshotTotal = 0;

        foreach (string path in selection.Raw.Sources)
        {
            CapturedFile file = CaptureFile(
                rootHandle,
                inventory,
                path,
                CapturedInputKind.Source,
                SourceFileBytesMaximum,
                "CSHARP_LIMIT_SOURCE_FILE_BYTES");
            sourceTotal = AddWithin(sourceTotal, file.SizeBytes, SourceTotalBytesMaximum, "CSHARP_LIMIT_SOURCE_TOTAL_BYTES");
            snapshotTotal = AddWithin(snapshotTotal, file.SizeBytes, SnapshotTotalBytesMaximum, "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES");
            captured.Add(file);
        }

        foreach (string path in selection.Raw.Contracts)
        {
            CapturedFile file = CaptureFile(
                rootHandle,
                inventory,
                path,
                CapturedInputKind.Contract,
                ContractFileBytesMaximum,
                "CSHARP_LIMIT_CONTRACT_FILE_BYTES");
            contractTotal = AddWithin(contractTotal, file.SizeBytes, ContractTotalBytesMaximum, "CSHARP_LIMIT_CONTRACT_TOTAL_BYTES");
            snapshotTotal = AddWithin(snapshotTotal, file.SizeBytes, SnapshotTotalBytesMaximum, "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES");
            captured.Add(file);
        }

        return new CapturedSnapshot(selection, captured.ToArray());
    }

    private static Dictionary<string, ExpectedNodeKind> BuildExpectedInventory(RawSelection selection)
    {
        var expected = new Dictionary<string, ExpectedNodeKind>(StringComparer.Ordinal);
        foreach (string path in selection.Sources)
        {
            AddExpectedFile(expected, path);
        }

        foreach (string path in selection.Contracts)
        {
            AddExpectedFile(expected, path);
        }

        return expected;
    }

    private static void AddExpectedFile(Dictionary<string, ExpectedNodeKind> expected, string path)
    {
        if (expected.ContainsKey(path))
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_PATH");
        }

        AddExpectedNodeWithinLimit(expected, path, ExpectedNodeKind.File);

        int separator = path.LastIndexOf('/');
        while (separator > 0)
        {
            string directory = path.Substring(0, separator);
            if (expected.TryGetValue(directory, out ExpectedNodeKind kind))
            {
                if (kind != ExpectedNodeKind.Directory)
                {
                    throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_PATH");
                }
            }
            else
            {
                AddExpectedNodeWithinLimit(expected, directory, ExpectedNodeKind.Directory);
            }

            separator = directory.LastIndexOf('/');
        }
    }

    private static void AddExpectedNodeWithinLimit(
        Dictionary<string, ExpectedNodeKind> expected,
        string path,
        ExpectedNodeKind kind)
    {
        if (expected.Count >= SnapshotEntriesMaximum)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_LIMIT_SNAPSHOT_ENTRIES");
        }

        expected.Add(path, kind);
    }

    private static Dictionary<string, InventoryNode> InspectInventory(
        SafeFileHandle root,
        Dictionary<string, ExpectedNodeKind> expected)
    {
        var observed = new Dictionary<string, InventoryNode>(StringComparer.Ordinal);
        var foldedPaths = new HashSet<string>(StringComparer.Ordinal);
        var fileIdentities = new HashSet<FileIdentity>();
        int observedEntries = 0;
        InspectDirectory(
            root,
            string.Empty,
            expected,
            observed,
            foldedPaths,
            fileIdentities,
            ref observedEntries);

        if (observed.Count != expected.Count)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
        }

        foreach (string path in expected.Keys)
        {
            if (!observed.ContainsKey(path))
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
            }
        }

        return observed;
    }

    private static void InspectDirectory(
        SafeFileHandle directory,
        string relativeDirectory,
        Dictionary<string, ExpectedNodeKind> expected,
        Dictionary<string, InventoryNode> observed,
        HashSet<string> foldedPaths,
        HashSet<FileIdentity> fileIdentities,
        ref int observedEntries)
    {
        List<string> names = ReadDirectoryNames(directory, ref observedEntries);
        names.Sort(StringComparer.Ordinal);
        foreach (string name in names)
        {
            string relativePath = relativeDirectory.Length == 0 ? name : relativeDirectory + "/" + name;
            if (relativePath.Length > 1_024)
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_LIMIT_NORMALIZED_PATH_BYTES");
            }

            if (!SelectionCodec.IsPortablePath(relativePath)
                || !foldedPaths.Add(relativePath.ToLowerInvariant()))
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_PATH");
            }

            if (!expected.TryGetValue(relativePath, out ExpectedNodeKind expectedKind))
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
            }

            UnixStat stat = FStatAtNoFollow(directory, name);
            if (expectedKind == ExpectedNodeKind.Directory)
            {
                if (!IsDirectory(stat.Mode))
                {
                    throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
                }

                using SafeFileHandle child = OpenAtNoFollow(directory, name, directory: true);
                if (!SameIdentityAndMetadata(stat, FStatChecked(child)))
                {
                    throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
                }

                observed.Add(relativePath, new InventoryNode(expectedKind, stat));
                InspectDirectory(
                    child,
                    relativePath,
                    expected,
                    observed,
                    foldedPaths,
                    fileIdentities,
                    ref observedEntries);
            }
            else
            {
                if (!IsRegular(stat.Mode) || stat.LinkCount != 1 || stat.Size < 0)
                {
                    throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
                }

                if (!fileIdentities.Add(new FileIdentity(stat.Device, stat.Inode)))
                {
                    throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
                }

                observed.Add(relativePath, new InventoryNode(expectedKind, stat));
            }
        }
    }

    private static List<string> ReadDirectoryNames(SafeFileHandle directory, ref int observedEntries)
    {
        int duplicate = NativeMethods.Dup(directory);
        if (duplicate < 0)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
        }

        using var duplicateHandle = new SafeFileHandle(new IntPtr(duplicate), ownsHandle: true);
        IntPtr stream = NativeMethods.FdOpenDir(duplicate);
        if (stream == IntPtr.Zero)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
        }

        duplicateHandle.SetHandleAsInvalid();
        var names = new List<string>();
        bool closeFailed = false;
        try
        {
            while (true)
            {
                Marshal.SetLastPInvokeError(0);
                IntPtr entry = NativeMethods.ReadDir(stream);
                if (entry == IntPtr.Zero)
                {
                    if (Marshal.GetLastPInvokeError() != 0)
                    {
                        throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
                    }

                    break;
                }

                string name = ReadDirectoryName(entry);
                if (name == "." || name == "..")
                {
                    continue;
                }

                if (observedEntries >= SnapshotEntriesMaximum)
                {
                    throw FrontendFailure.Rejected("capture", "CSHARP_LIMIT_SNAPSHOT_ENTRIES");
                }

                observedEntries = checked(observedEntries + 1);
                names.Add(name);
            }
        }
        finally
        {
            closeFailed = NativeMethods.CloseDir(stream) != 0;
        }

        if (closeFailed)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
        }

        return names;
    }

    private static string ReadDirectoryName(IntPtr entry)
    {
        int recordLength = unchecked((ushort)Marshal.ReadInt16(entry, NativeMethods.DirectoryEntryRecordLengthOffset));
        int capacity = recordLength - NativeMethods.DirectoryEntryNameOffset;
        if (capacity <= 0 || capacity > 512)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_PATH");
        }

        var characters = new char[capacity];
        int length = 0;
        while (length < capacity)
        {
            byte value = Marshal.ReadByte(entry, NativeMethods.DirectoryEntryNameOffset + length);
            if (value == 0)
            {
                break;
            }

            if (value > 0x7f)
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_PATH");
            }

            characters[length] = (char)value;
            length++;
        }

        if (length == 0 || length == capacity || length > 255)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_PATH");
        }

        return new string(characters, 0, length);
    }

    private static CapturedFile CaptureFile(
        SafeFileHandle root,
        Dictionary<string, InventoryNode> inventory,
        string normalizedPath,
        CapturedInputKind kind,
        int fileBytesMaximum,
        string fileLimitCode)
    {
        InventoryNode fileInventory = inventory[normalizedPath];
        if (fileInventory.Kind != ExpectedNodeKind.File)
        {
            throw FrontendFailure.Internal("capture");
        }

        string[] components = normalizedPath.Split('/');
        SafeFileHandle? ownedDirectory = null;
        SafeFileHandle directory = root;
        string prefix = string.Empty;
        try
        {
            for (int index = 0; index < components.Length - 1; index++)
            {
                prefix = prefix.Length == 0 ? components[index] : prefix + "/" + components[index];
                SafeFileHandle child = OpenAtNoFollow(directory, components[index], directory: true);
                UnixStat observedDirectory = FStatChecked(child);
                if (!inventory.TryGetValue(prefix, out InventoryNode expectedDirectory)
                    || expectedDirectory.Kind != ExpectedNodeKind.Directory
                    || !SameIdentityAndMetadata(expectedDirectory.Stat, observedDirectory))
                {
                    child.Dispose();
                    throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
                }

                ownedDirectory?.Dispose();
                ownedDirectory = child;
                directory = child;
            }

            using SafeFileHandle handle = OpenAtNoFollow(directory, components[^1], directory: false);
            return ReadCapturedFile(handle, normalizedPath, kind, fileInventory, fileBytesMaximum, fileLimitCode);
        }
        finally
        {
            ownedDirectory?.Dispose();
        }
    }

    private static CapturedFile ReadCapturedFile(
        SafeFileHandle handle,
        string normalizedPath,
        CapturedInputKind kind,
        InventoryNode inventory,
        int fileBytesMaximum,
        string fileLimitCode)
    {
        UnixStat before = FStatChecked(handle);
        if (!SameIdentityAndMetadata(inventory.Stat, before)
            || !IsRegular(before.Mode)
            || before.LinkCount != 1)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
        }

        if (before.Size < 0 || before.Size > fileBytesMaximum)
        {
            throw FrontendFailure.Rejected("capture", fileLimitCode);
        }

        int size = checked((int)before.Size);
        var bytes = new byte[size];
        using (var stream = new FileStream(handle, FileAccess.Read, bufferSize: 8_192, isAsync: false))
        {
            int offset = 0;
            while (offset < bytes.Length)
            {
                int count = stream.Read(bytes, offset, bytes.Length - offset);
                if (count == 0)
                {
                    throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
                }

                offset = checked(offset + count);
            }

            if (stream.ReadByte() != -1)
            {
                throw FrontendFailure.Rejected(
                    "capture",
                    size == fileBytesMaximum ? fileLimitCode : "CSHARP_CAPTURE_INVENTORY");
            }

            UnixStat after = FStatChecked(handle);
            if (!SameIdentityAndMetadata(before, after) || after.Size != bytes.Length)
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
            }
        }

        return new CapturedFile(kind, normalizedPath, bytes);
    }

    private static void ValidateProjectedLimits(
        RawSelection selection,
        Dictionary<string, InventoryNode> inventory)
    {
        ulong sourceTotal = 0;
        ulong contractTotal = 0;
        ulong snapshotTotal = 0;
        foreach (string path in selection.Sources)
        {
            ulong size = ProjectedFileSize(inventory[path], SourceFileBytesMaximum, "CSHARP_LIMIT_SOURCE_FILE_BYTES");
            sourceTotal = AddWithin(sourceTotal, size, SourceTotalBytesMaximum, "CSHARP_LIMIT_SOURCE_TOTAL_BYTES");
            snapshotTotal = AddWithin(snapshotTotal, size, SnapshotTotalBytesMaximum, "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES");
        }

        foreach (string path in selection.Contracts)
        {
            ulong size = ProjectedFileSize(inventory[path], ContractFileBytesMaximum, "CSHARP_LIMIT_CONTRACT_FILE_BYTES");
            contractTotal = AddWithin(contractTotal, size, ContractTotalBytesMaximum, "CSHARP_LIMIT_CONTRACT_TOTAL_BYTES");
            snapshotTotal = AddWithin(snapshotTotal, size, SnapshotTotalBytesMaximum, "CSHARP_LIMIT_SNAPSHOT_TOTAL_BYTES");
        }
    }

    private static ulong ProjectedFileSize(InventoryNode inventory, int maximum, string code)
    {
        if (inventory.Kind != ExpectedNodeKind.File || inventory.Stat.Size < 0)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
        }

        ulong size = checked((ulong)inventory.Stat.Size);
        if (size > checked((ulong)maximum))
        {
            throw FrontendFailure.Rejected("capture", code);
        }

        return size;
    }

    private static ulong AddWithin(ulong current, int increment, int maximum, string code)
    {
        return AddWithin(current, checked((ulong)increment), maximum, code);
    }

    private static ulong AddWithin(ulong current, ulong increment, int maximum, string code)
    {
        ulong next;
        try
        {
            next = checked(current + increment);
        }
        catch (OverflowException)
        {
            throw FrontendFailure.Rejected("capture", code);
        }

        if (next > checked((ulong)maximum))
        {
            throw FrontendFailure.Rejected("capture", code);
        }

        return next;
    }

    private static SafeFileHandle OpenAbsoluteDirectoryNoFollow(string absolutePath)
    {
        int descriptor = NativeMethods.Open(
            "/",
            NativeMethods.OpenReadOnly
                | NativeMethods.OpenDirectory
                | NativeMethods.OpenCloseOnExec
                | NativeMethods.OpenNoFollow);
        if (descriptor < 0)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
        }

        var current = new SafeFileHandle(new IntPtr(descriptor), ownsHandle: true);
        try
        {
            foreach (string component in absolutePath.Split('/', StringSplitOptions.RemoveEmptyEntries))
            {
                SafeFileHandle child = OpenAtNoFollow(current, component, directory: true);
                current.Dispose();
                current = child;
            }

            SafeFileHandle result = current;
            current = new SafeFileHandle(IntPtr.Zero, ownsHandle: false);
            return result;
        }
        finally
        {
            current.Dispose();
        }
    }

    private static SafeFileHandle OpenAtNoFollow(
        SafeFileHandle directoryHandle,
        string name,
        bool directory)
    {
        int flags = NativeMethods.OpenReadOnly | NativeMethods.OpenCloseOnExec | NativeMethods.OpenNoFollow;
        if (directory)
        {
            flags |= NativeMethods.OpenDirectory;
        }
        else
        {
            flags |= NativeMethods.OpenNonBlocking;
        }

        int descriptor = NativeMethods.OpenAt(directoryHandle, name, flags);
        if (descriptor < 0)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
        }

        return new SafeFileHandle(new IntPtr(descriptor), ownsHandle: true);
    }

    private static UnixStat FStatAtNoFollow(SafeFileHandle directory, string name)
    {
        if (NativeMethods.FStatAt(directory, name, out UnixStat stat, NativeMethods.AtSymlinkNoFollow) != 0)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_INVENTORY");
        }

        return stat;
    }

    private static UnixStat FStatChecked(SafeFileHandle handle)
    {
        if (NativeMethods.FStat(handle, out UnixStat stat) != 0)
        {
            throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_FILE_TYPE");
        }

        return stat;
    }

    private static bool SameIdentityAndMetadata(UnixStat left, UnixStat right)
    {
        return left.Device == right.Device
            && left.Inode == right.Inode
            && left.LinkCount == right.LinkCount
            && left.Mode == right.Mode
            && left.Size == right.Size
            && left.Modified.Seconds == right.Modified.Seconds
            && left.Modified.Nanoseconds == right.Modified.Nanoseconds
            && left.Changed.Seconds == right.Changed.Seconds
            && left.Changed.Nanoseconds == right.Changed.Nanoseconds;
    }

    private static bool IsRegular(uint mode) => (mode & NativeMethods.FileTypeMask) == NativeMethods.RegularFile;

    private static bool IsDirectory(uint mode) => (mode & NativeMethods.FileTypeMask) == NativeMethods.Directory;

    private enum ExpectedNodeKind
    {
        Directory,
        File,
    }

    private readonly struct InventoryNode
    {
        internal InventoryNode(ExpectedNodeKind kind, UnixStat stat)
        {
            Kind = kind;
            Stat = stat;
        }

        internal ExpectedNodeKind Kind { get; }

        internal UnixStat Stat { get; }
    }

    private readonly struct FileIdentity : IEquatable<FileIdentity>
    {
        internal FileIdentity(ulong device, ulong inode)
        {
            Device = device;
            Inode = inode;
        }

        private ulong Device { get; }

        private ulong Inode { get; }

        public bool Equals(FileIdentity other) => Device == other.Device && Inode == other.Inode;

        public override bool Equals(object? value) => value is FileIdentity other && Equals(other);

        public override int GetHashCode() => HashCode.Combine(Device, Inode);
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct UnixTimespec
    {
        internal long Seconds;
        internal long Nanoseconds;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct UnixStat
    {
        internal ulong Device;
        internal ulong Inode;
        internal ulong LinkCount;
        internal uint Mode;
        internal uint UserId;
        internal uint GroupId;
        internal int Padding;
        internal ulong SpecialDevice;
        internal long Size;
        internal long BlockSize;
        internal long Blocks;
        internal UnixTimespec Accessed;
        internal UnixTimespec Modified;
        internal UnixTimespec Changed;
        internal long Reserved0;
        internal long Reserved1;
        internal long Reserved2;
    }

    private static class NativeMethods
    {
        internal const int OpenReadOnly = 0;
        internal const int OpenNonBlocking = 0x800;
        internal const int OpenDirectory = 0x10_000;
        internal const int OpenNoFollow = 0x20_000;
        internal const int OpenCloseOnExec = 0x80_000;
        internal const int AtSymlinkNoFollow = 0x100;
        internal const int DirectoryEntryRecordLengthOffset = 16;
        internal const int DirectoryEntryNameOffset = 19;
        internal const uint FileTypeMask = 0xf000;
        internal const uint Directory = 0x4000;
        internal const uint RegularFile = 0x8000;

        [DllImport("libc", EntryPoint = "fstat", SetLastError = true)]
        internal static extern int FStat(SafeFileHandle descriptor, out UnixStat stat);

        [DllImport("libc", EntryPoint = "fstatat", SetLastError = true)]
        internal static extern int FStatAt(
            SafeFileHandle directory,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            out UnixStat stat,
            int flags);

        [DllImport("libc", EntryPoint = "open", SetLastError = true)]
        internal static extern int Open([MarshalAs(UnmanagedType.LPUTF8Str)] string path, int flags);

        [DllImport("libc", EntryPoint = "openat", SetLastError = true)]
        internal static extern int OpenAt(
            SafeFileHandle directory,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            int flags);

        [DllImport("libc", EntryPoint = "dup", SetLastError = true)]
        internal static extern int Dup(SafeFileHandle descriptor);

        [DllImport("libc", EntryPoint = "fdopendir", SetLastError = true)]
        internal static extern IntPtr FdOpenDir(int descriptor);

        [DllImport("libc", EntryPoint = "readdir", SetLastError = true)]
        internal static extern IntPtr ReadDir(IntPtr directory);

        [DllImport("libc", EntryPoint = "closedir", SetLastError = true)]
        internal static extern int CloseDir(IntPtr directory);
    }
}
