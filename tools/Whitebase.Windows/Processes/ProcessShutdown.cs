using System.ComponentModel;
using System.Diagnostics;
using Whitebase.Windows.Desktop;

namespace Whitebase.Windows.Processes;

/// <summary>
/// プロセス停止処理の結果を表します。
/// </summary>
public enum ProcessShutdownStatus
{
    /// <summary>
    /// 停止要求前にプロセスが終了していました。
    /// </summary>
    AlreadyExited,

    /// <summary>
    /// ウィンドウへの終了要求によって正常終了しました。
    /// </summary>
    ClosedGracefully,

    /// <summary>
    /// プロセスツリーを強制終了しました。
    /// </summary>
    TerminatedForcefully,

    /// <summary>
    /// 制限時間内に終了しませんでした。
    /// </summary>
    TimedOut,
}

/// <summary>
/// プロセス停止処理の結果です。
/// </summary>
/// <param name="Status">停止状態。</param>
/// <param name="ApplicationFound">
/// 対象アプリケーションが見つかったかどうか。
/// </param>
/// <param name="ApplicationProcessId">
/// 対象アプリケーションのプロセスID。
/// </param>
/// <param name="CloseRequested">
/// WM_CLOSEを送信できたかどうか。
/// </param>
/// <param name="GracefulShutdownError">
/// 正常終了処理中に発生した警告。
/// </param>
public sealed record ProcessShutdownResult(
    ProcessShutdownStatus Status,
    bool ApplicationFound,
    int? ApplicationProcessId,
    bool CloseRequested,
    string? GracefulShutdownError);

/// <summary>
/// Windowsプロセスの正常終了と強制終了を制御します。
/// </summary>
public static class ProcessShutdown
{
    /// <summary>
    /// 子孫にあるGUIアプリケーションの正常終了を試し、
    /// 終了しなかった場合はプロセスツリーを強制終了します。
    /// </summary>
    /// <param name="rootProcess">
    /// ops.batなどを実行しているルートプロセス。
    /// </param>
    /// <param name="applicationProcessName">
    /// 正常終了を要求するGUIプロセス名。
    /// </param>
    /// <param name="gracefulTimeout">
    /// 正常終了を待機する時間。
    /// </param>
    /// <param name="forceTimeout">
    /// 強制終了後に待機する時間。
    /// </param>
    /// <param name="cancellationToken">
    /// 停止処理をキャンセルするトークン。
    /// </param>
    /// <returns>停止処理の結果。</returns>
    public static async Task<ProcessShutdownResult> StopAsync(
    Process rootProcess,
    string applicationProcessName,
    TimeSpan gracefulTimeout,
    TimeSpan forceTimeout,
    CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(rootProcess);

        ArgumentException.ThrowIfNullOrWhiteSpace(
            applicationProcessName);

        ValidateTimeout(
            gracefulTimeout,
            nameof(gracefulTimeout));

        ValidateTimeout(
            forceTimeout,
            nameof(forceTimeout));

        bool rootWasAlreadyExited =
            HasExited(rootProcess);

        bool applicationFound = false;
        bool closeRequested = false;
        int? applicationProcessId = null;
        string? gracefulShutdownError = null;

        // WM_CLOSEを送る前に、現在のプロセスツリーを記録する。
        // 親が先に終了しても、記録したPIDを使って掃除できる。
        var trackedProcessIds =
            new HashSet<int>
            {
            rootProcess.Id,
            };

        if (!rootWasAlreadyExited)
        {
            try
            {
                foreach (int descendantProcessId in
                         ProcessTree.GetDescendantProcessIds(
                             rootProcess.Id))
                {
                    trackedProcessIds.Add(descendantProcessId);
                }
            }
            catch (Win32Exception exception)
            {
                gracefulShutdownError =
                    CombineErrors(
                        gracefulShutdownError,
                        exception.Message);
            }
        }

        try
        {
            using Process? applicationProcess =
                ProcessTree.FindDescendant(
                    rootProcess.Id,
                    applicationProcessName);

            if (applicationProcess is not null)
            {
                applicationFound = true;
                applicationProcessId =
                    applicationProcess.Id;

                trackedProcessIds.Add(
                    applicationProcess.Id);

                if (!HasExited(applicationProcess))
                {
                    closeRequested =
                        WindowCloser.RequestClose(
                            applicationProcess.Id);
                }

                if (closeRequested)
                {
                    // WM_CLOSEでウィンドウはすぐ閉じる。
                    // その後、アプリ本体も終了するか待機する。
                    bool applicationExited =
                        await WaitForExitAsync(
                            applicationProcess,
                            gracefulTimeout,
                            cancellationToken);

                    bool rootExited =
                        await WaitForExitAsync(
                            rootProcess,
                            TimeSpan.FromSeconds(1),
                            cancellationToken);

                    if (applicationExited &&
                        rootExited)
                    {
                        return new ProcessShutdownResult(
                            ProcessShutdownStatus.ClosedGracefully,
                            applicationFound,
                            applicationProcessId,
                            closeRequested,
                            gracefulShutdownError);
                    }
                }
            }
        }
        catch (OperationCanceledException)
            when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (Exception exception)
            when (exception is Win32Exception
                or ArgumentException
                or InvalidOperationException)
        {
            // 正常終了処理が失敗しても、
            // 後続の強制終了処理を続ける。
            gracefulShutdownError =
                CombineErrors(
                    gracefulShutdownError,
                    exception.Message);
        }

        // STOPを押した時点でルートが既に終了しており、
        // 対象アプリも見つからなかった。
        if (rootWasAlreadyExited &&
            !applicationFound)
        {
            return new ProcessShutdownResult(
                ProcessShutdownStatus.AlreadyExited,
                ApplicationFound: false,
                ApplicationProcessId: null,
                CloseRequested: false,
                GracefulShutdownError:
                    gracefulShutdownError);
        }

        bool applicationStillRunning =
            applicationProcessId is int applicationPid &&
            IsProcessRunning(applicationPid);

        // ルートと対象アプリの両方が終了していれば正常終了。
        if (HasExited(rootProcess) &&
            !applicationStillRunning)
        {
            return new ProcessShutdownResult(
                ProcessShutdownStatus.ClosedGracefully,
                applicationFound,
                applicationProcessId,
                closeRequested,
                gracefulShutdownError);
        }

        // ルートが生存中ならプロセスツリーを強制終了する。
        if (!HasExited(rootProcess))
        {
            try
            {
                rootProcess.Kill(
                    entireProcessTree: true);
            }
            catch (InvalidOperationException)
            {
                // 確認直後にルートが終了した。
                // 記録済みPIDの掃除は引き続き実行する。
            }
            catch (Win32Exception exception)
            {
                gracefulShutdownError =
                    CombineErrors(
                        gracefulShutdownError,
                        exception.Message);
            }
        }

        // Kill(entireProcessTree: true)から漏れたプロセスや、
        // 親終了後に残ったプロセスを個別に停止する。
        foreach (int trackedProcessId in trackedProcessIds)
        {
            if (trackedProcessId != rootProcess.Id)
            {
                TryKillProcess(trackedProcessId);
            }
        }

        bool allProcessesExited =
            await WaitForProcessesToExitAsync(
                trackedProcessIds,
                forceTimeout,
                cancellationToken);

        return new ProcessShutdownResult(
            allProcessesExited
                ? ProcessShutdownStatus.TerminatedForcefully
                : ProcessShutdownStatus.TimedOut,
            applicationFound,
            applicationProcessId,
            closeRequested,
            gracefulShutdownError);
    }

    /// <summary>
    /// 指定された時間内にプロセスが終了することを待機します。
    /// </summary>
    private static async Task<bool> WaitForExitAsync(
        Process process,
        TimeSpan timeout,
        CancellationToken cancellationToken)
    {
        if (HasExited(process))
        {
            return true;
        }

        using var timeoutSource =
            new CancellationTokenSource(timeout);

        using var linkedSource =
            CancellationTokenSource.CreateLinkedTokenSource(
                cancellationToken,
                timeoutSource.Token);

        try
        {
            await process.WaitForExitAsync(
                linkedSource.Token);

            return true;
        }
        catch (OperationCanceledException)
            when (
                timeoutSource.IsCancellationRequested &&
                !cancellationToken.IsCancellationRequested)
        {
            return HasExited(process);
        }
    }

    /// <summary>
    /// 指定されたプロセスを強制終了します。
    /// </summary>
    private static void TryKillProcess(int processId)
    {
        try
        {
            using Process process =
                Process.GetProcessById(processId);

            if (!process.HasExited)
            {
                process.Kill(
                    entireProcessTree: true);
            }
        }
        catch (ArgumentException)
        {
            // 既に終了している。
        }
        catch (InvalidOperationException)
        {
            // 既に終了している。
        }
        catch (Win32Exception)
        {
            // アクセスできないプロセスは、
            // 後続の終了確認へ任せる。
        }
    }

    /// <summary>
    /// 指定されたPIDのプロセスがすべて終了することを待機します。
    /// </summary>
    private static async Task<bool>
        WaitForProcessesToExitAsync(
            IEnumerable<int> processIds,
            TimeSpan timeout,
            CancellationToken cancellationToken)
    {
        var remainingProcessIds =
            new HashSet<int>(processIds);

        var stopwatch =
            Stopwatch.StartNew();

        while (remainingProcessIds.Count > 0 &&
               stopwatch.Elapsed < timeout)
        {
            cancellationToken.ThrowIfCancellationRequested();

            remainingProcessIds.RemoveWhere(
                processId =>
                    !IsProcessRunning(processId));

            if (remainingProcessIds.Count == 0)
            {
                return true;
            }

            await Task.Delay(
                TimeSpan.FromMilliseconds(100),
                cancellationToken);
        }

        remainingProcessIds.RemoveWhere(
            processId =>
                !IsProcessRunning(processId));

        return remainingProcessIds.Count == 0;
    }

    /// <summary>
    /// Processオブジェクトが終了済みか確認します。
    /// </summary>
    private static bool HasExited(Process process)
    {
        try
        {
            return process.HasExited;
        }
        catch (ArgumentException)
        {
            return true;
        }
        catch (InvalidOperationException)
        {
            return true;
        }
        catch (Win32Exception)
        {
            return true;
        }
    }

    /// <summary>
    /// 指定されたPIDのプロセスが実行中か確認します。
    /// </summary>
    private static bool IsProcessRunning(int processId)
    {
        try
        {
            using Process process =
                Process.GetProcessById(processId);

            return !process.HasExited;
        }
        catch (ArgumentException)
        {
            return false;
        }
        catch (InvalidOperationException)
        {
            return false;
        }
        catch (Win32Exception)
        {
            return true;
        }
    }

    /// <summary>
    /// タイムアウト値を検証します。
    /// </summary>
    private static void ValidateTimeout(
        TimeSpan timeout,
        string parameterName)
    {
        if (timeout <= TimeSpan.Zero)
        {
            throw new ArgumentOutOfRangeException(
                parameterName,
                timeout,
                "Timeout must be greater than zero.");
        }
    }

    /// <summary>
    /// エラーメッセージを結合します。
    /// </summary>
    private static string CombineErrors(
        string? currentError,
        string nextError)
    {
        return string.IsNullOrWhiteSpace(currentError)
            ? nextError
            : $"{currentError} / {nextError}";
    }

    /// <summary>
    /// 指定されたプロセスの可視ウィンドウが消えるまで待機します。
    /// </summary>
    private static async Task<bool>
        WaitForWindowToCloseAsync(
            int processId,
            TimeSpan timeout,
            CancellationToken cancellationToken)
    {
        var stopwatch = Stopwatch.StartNew();

        while (stopwatch.Elapsed < timeout)
        {
            cancellationToken.ThrowIfCancellationRequested();

            if (!WindowCloser.HasVisibleTopLevelWindow(
                    processId))
            {
                return true;
            }

            await Task.Delay(
                TimeSpan.FromMilliseconds(100),
                cancellationToken);
        }

        return !WindowCloser.HasVisibleTopLevelWindow(
            processId);
    }
}