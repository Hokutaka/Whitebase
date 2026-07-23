using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Whitebase.Windows.Interop;

namespace Whitebase.Windows.Processes;

/// <summary>
/// Windows上の親子プロセス関係を探索します。
/// </summary>
public static class ProcessTree
{
    /// <summary>
    /// プロセス列挙の終端を表すWin32エラーコードです。
    /// </summary>
    private const int ErrorNoMoreFiles = 18;

    /// <summary>
    /// 指定されたルートプロセスの子孫から、
    /// 指定名のプロセスを検索します。
    /// </summary>
    /// <param name="rootProcessId">
    /// 探索を開始するルートプロセスのID。
    /// </param>
    /// <param name="processName">
    /// 検索するプロセス名。拡張子は省略可能です。
    /// </param>
    /// <returns>
    /// 見つかったプロセス。
    /// 見つからなかった場合はnull。
    /// </returns>
    public static Process? FindDescendant(
        int rootProcessId,
        string processName)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(
            rootProcessId);

        ArgumentException.ThrowIfNullOrWhiteSpace(
            processName);

        string targetName =
            Path.GetFileNameWithoutExtension(processName);

        IReadOnlyList<SnapshotProcess> snapshot =
            CaptureSnapshot();

        foreach (SnapshotProcess process
                 in EnumerateDescendants(
                     rootProcessId,
                     snapshot))
        {
            string currentName =
                Path.GetFileNameWithoutExtension(
                    process.Name);

            if (!string.Equals(
                    currentName,
                    targetName,
                    StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            try
            {
                return Process.GetProcessById(
                    process.ProcessId);
            }
            catch (ArgumentException)
            {
                // スナップショット取得後に終了した。
            }
            catch (InvalidOperationException)
            {
                // 終了処理中などで取得できなかった。
            }
        }

        return null;
    }

    /// <summary>
    /// 指定されたルートプロセスの子孫PIDを取得します。
    /// </summary>
    /// <param name="rootProcessId">
    /// 探索を開始するルートプロセスのID。
    /// </param>
    /// <returns>子孫プロセスのID一覧。</returns>
    public static IReadOnlyList<int> GetDescendantProcessIds(
        int rootProcessId)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(
            rootProcessId);

        IReadOnlyList<SnapshotProcess> snapshot =
            CaptureSnapshot();

        return EnumerateDescendants(
                rootProcessId,
                snapshot)
            .Select(process => process.ProcessId)
            .ToArray();
    }

    /// <summary>
    /// 現在実行中のプロセス情報を取得します。
    /// </summary>
    /// <returns>プロセス情報のスナップショット。</returns>
    private static IReadOnlyList<SnapshotProcess>
        CaptureSnapshot()
    {
        nint snapshotHandle =
            NativeMethods.CreateToolhelp32Snapshot(
                NativeMethods.Th32CsSnapProcess,
                processId: 0);

        if (snapshotHandle ==
            NativeMethods.InvalidHandleValue)
        {
            throw new Win32Exception(
                Marshal.GetLastWin32Error());
        }

        try
        {
            var entry =
                new NativeMethods.ProcessEntry32
                {
                    Size =
                        (uint)Marshal.SizeOf<
                            NativeMethods.ProcessEntry32>(),

                    ExecutableFile = string.Empty,
                };

            if (!NativeMethods.Process32First(
                    snapshotHandle,
                    ref entry))
            {
                int errorCode =
                    Marshal.GetLastWin32Error();

                if (errorCode == ErrorNoMoreFiles)
                {
                    return [];
                }

                throw new Win32Exception(errorCode);
            }

            var processes =
                new List<SnapshotProcess>();

            while (true)
            {
                processes.Add(
                    new SnapshotProcess(
                        unchecked((int)entry.ProcessId),
                        unchecked(
                            (int)entry.ParentProcessId),
                        entry.ExecutableFile));

                if (NativeMethods.Process32Next(
                        snapshotHandle,
                        ref entry))
                {
                    continue;
                }

                int errorCode =
                    Marshal.GetLastWin32Error();

                if (errorCode != ErrorNoMoreFiles)
                {
                    throw new Win32Exception(errorCode);
                }

                break;
            }

            return processes;
        }
        finally
        {
            _ = NativeMethods.CloseHandle(
                snapshotHandle);
        }
    }

    /// <summary>
    /// 指定されたルートプロセスの子孫を幅優先で列挙します。
    /// </summary>
    /// <param name="rootProcessId">
    /// 探索を開始するルートプロセスのID。
    /// </param>
    /// <param name="snapshot">
    /// プロセス情報のスナップショット。
    /// </param>
    /// <returns>子孫プロセス。</returns>
    private static IEnumerable<SnapshotProcess>
        EnumerateDescendants(
            int rootProcessId,
            IReadOnlyList<SnapshotProcess> snapshot)
    {
        var childrenByParent =
            new Dictionary<int, List<SnapshotProcess>>();

        foreach (SnapshotProcess process in snapshot)
        {
            if (!childrenByParent.TryGetValue(
                    process.ParentProcessId,
                    out List<SnapshotProcess>? children))
            {
                children = [];

                childrenByParent[
                    process.ParentProcessId] = children;
            }

            children.Add(process);
        }

        var pending = new Queue<int>();

        var visited =
            new HashSet<int>
            {
                rootProcessId,
            };

        pending.Enqueue(rootProcessId);

        while (pending.TryDequeue(
                   out int parentProcessId))
        {
            if (!childrenByParent.TryGetValue(
                    parentProcessId,
                    out List<SnapshotProcess>? children))
            {
                continue;
            }

            foreach (SnapshotProcess child in children)
            {
                if (!visited.Add(child.ProcessId))
                {
                    continue;
                }

                yield return child;
                pending.Enqueue(child.ProcessId);
            }
        }
    }

    /// <summary>
    /// プロセススナップショット内の1プロセスを表します。
    /// </summary>
    /// <param name="ProcessId">プロセスID。</param>
    /// <param name="ParentProcessId">親プロセスID。</param>
    /// <param name="Name">実行ファイル名。</param>
    private sealed record SnapshotProcess(
        int ProcessId,
        int ParentProcessId,
        string Name);
}