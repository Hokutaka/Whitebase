using System.ComponentModel;
using System.Runtime.InteropServices;
using Whitebase.Windows.Interop;

namespace Whitebase.Windows.Desktop;

/// <summary>
/// Windowsデスクトップアプリケーションの
/// ウィンドウへ終了要求を送信します。
/// </summary>
public static class WindowCloser
{
    /// <summary>
    /// 指定されたプロセスが所有する可視トップレベル
    /// ウィンドウへWM_CLOSEを送信します。
    /// </summary>
    /// <param name="processId">
    /// 対象プロセスのID。
    /// </param>
    /// <returns>
    /// 1つ以上のウィンドウへ終了要求を
    /// 送信できた場合はtrue。
    /// </returns>
    public static bool RequestClose(int processId)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(
            processId);

        IReadOnlyList<nint> windowHandles =
            FindTopLevelWindows(processId);

        bool closeRequested = false;

        foreach (nint windowHandle in windowHandles)
        {
            bool posted =
                NativeMethods.PostMessage(
                    windowHandle,
                    NativeMethods.WmClose,
                    nint.Zero,
                    nint.Zero);

            closeRequested |= posted;
        }

        return closeRequested;
    }

    /// <summary>
    /// 指定されたプロセスが所有する可視トップレベル
    /// ウィンドウを取得します。
    /// </summary>
    /// <param name="processId">
    /// 対象プロセスのID。
    /// </param>
    /// <returns>
    /// 対象プロセスが所有するウィンドウハンドルの一覧。
    /// </returns>
    private static IReadOnlyList<nint> FindTopLevelWindows(
        int processId)
    {
        var windowHandles = new List<nint>();

        NativeMethods.EnumWindowsCallback callback =
            (windowHandle, parameter) =>
            {
                _ = parameter;

                _ = NativeMethods.GetWindowThreadProcessId(
                    windowHandle,
                    out uint ownerProcessId);

                if (ownerProcessId == (uint)processId &&
                    NativeMethods.IsWindowVisible(windowHandle))
                {
                    windowHandles.Add(windowHandle);
                }

                return true;
            };

        bool succeeded =
            NativeMethods.EnumWindows(
                callback,
                nint.Zero);

        // ネイティブ側で列挙が終わるまで、
        // デリゲートがGCされないことを保証する。
        GC.KeepAlive(callback);

        if (!succeeded)
        {
            throw new Win32Exception(
                Marshal.GetLastWin32Error());
        }

        return windowHandles;
    }

    /// <summary>
    /// 指定されたプロセスが可視トップレベルウィンドウを
    /// 所有しているか確認します。
    /// </summary>
    /// <param name="processId">対象プロセスのID。</param>
    /// <returns>
    /// 可視トップレベルウィンドウが存在する場合はtrue。
    /// </returns>
    public static bool HasVisibleTopLevelWindow(
        int processId)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(
            processId);

        return FindTopLevelWindows(processId).Count > 0;
    }
}