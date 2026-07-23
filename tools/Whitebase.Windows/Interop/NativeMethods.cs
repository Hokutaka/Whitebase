using System.Runtime.InteropServices;
namespace Whitebase.Windows.Interop;

/// <summary>
/// Whitebase.Windows内部で使用するWindows APIを定義します。
/// </summary>
internal static class NativeMethods
{
    /// <summary>
    /// プロセス一覧を含むスナップショットを作成します。
    /// </summary>
    internal const uint Th32CsSnapProcess = 0x00000002;

    /// <summary>
    /// ウィンドウへ終了要求を送るメッセージです。
    /// </summary>
    internal const uint WmClose = 0x0010;

    /// <summary>
    /// CreateToolhelp32Snapshotが失敗した場合の戻り値です。
    /// </summary>
    internal static readonly nint InvalidHandleValue =
        new(-1);

    /// <summary>
    /// EnumWindowsで使用するコールバックです。
    /// </summary>
    /// <param name="windowHandle">
    /// 列挙されたトップレベルウィンドウのハンドル。
    /// </param>
    /// <param name="parameter">
    /// 呼び出し元から渡されたパラメーター。
    /// </param>
    /// <returns>
    /// 列挙を継続する場合はtrue。
    /// </returns>
    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal delegate bool EnumWindowsCallback(
        nint windowHandle,
        nint parameter);

    /// <summary>
    /// Tool Help APIから取得するプロセス情報です。
    /// </summary>
    [StructLayout(
        LayoutKind.Sequential,
        CharSet = CharSet.Unicode)]
    internal struct ProcessEntry32
    {
        internal uint Size;
        internal uint Usage;
        internal uint ProcessId;
        internal nuint DefaultHeapId;
        internal uint ModuleId;
        internal uint ThreadCount;
        internal uint ParentProcessId;
        internal int BasePriority;
        internal uint Flags;

        [MarshalAs(
            UnmanagedType.ByValTStr,
            SizeConst = 260)]
        internal string ExecutableFile;
    }

    /// <summary>
    /// 実行中プロセスのスナップショットを作成します。
    /// </summary>
    [DllImport(
        "kernel32.dll",
        SetLastError = true)]
    internal static extern nint CreateToolhelp32Snapshot(
        uint flags,
        uint processId);

    /// <summary>
    /// スナップショット内の最初のプロセスを取得します。
    /// </summary>
    [DllImport(
        "kernel32.dll",
        EntryPoint = "Process32FirstW",
        CharSet = CharSet.Unicode,
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool Process32First(
        nint snapshotHandle,
        ref ProcessEntry32 processEntry);

    /// <summary>
    /// スナップショット内の次のプロセスを取得します。
    /// </summary>
    [DllImport(
        "kernel32.dll",
        EntryPoint = "Process32NextW",
        CharSet = CharSet.Unicode,
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool Process32Next(
        nint snapshotHandle,
        ref ProcessEntry32 processEntry);

    /// <summary>
    /// Windowsハンドルを閉じます。
    /// </summary>
    [DllImport(
        "kernel32.dll",
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool CloseHandle(
        nint handle);

    /// <summary>
    /// デスクトップ上のトップレベルウィンドウを列挙します。
    /// </summary>
    [DllImport(
        "user32.dll",
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool EnumWindows(
        EnumWindowsCallback callback,
        nint parameter);

    /// <summary>
    /// 指定されたウィンドウの所有プロセスIDを取得します。
    /// </summary>
    /// <param name="windowHandle">対象ウィンドウのハンドル。</param>
    /// <param name="processId">所有プロセスのIDを格納する変数。</param>
    /// <returns></returns>
    [DllImport("user32.dll")]
    internal static extern uint GetWindowThreadProcessId(
        nint windowHandle,
        out uint processId);

    /// <summary>
    /// ウィンドウが表示されているか確認します。
    /// </summary>
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool IsWindowVisible(
        nint windowHandle);

    /// <summary>
    /// ウィンドウのメッセージキューへメッセージを送ります。
    /// </summary>
    [DllImport(
        "user32.dll",
        EntryPoint = "PostMessageW",
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool PostMessage(
        nint windowHandle,
        uint message,
        nint wParam,
        nint lParam);
}