using System.Diagnostics;
using System.IO;
using System.Management;
using System.Text;
using System.Text.RegularExpressions;
using System.Windows;
using System.Windows.Controls;
using Whitebase.Windows.Desktop;
using Whitebase.Windows.Processes;

namespace Whitebase.ControlPanel;

public partial class MainWindow : Window
{
    /// <summary>
    /// ANSIエスケープシーケンスを検出する正規表現。
    /// </summary>
    private static readonly Regex AnsiEscapeRegex = new(
    @"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])",
    RegexOptions.Compiled);

    /// <summary>
    /// 許可されているコマンドのセット。
    /// </summary>
    private static readonly HashSet<string> AllowedCommands =
        new(StringComparer.Ordinal)
        {
            "setup",
            "test",
            "fmt",
            "lint",
            "check",
            "cpp-check",
            "cpp-backend-check",
            "cpp-adapter-check",
            "wasm-check",
            "asm-check",
            "tree",
            "diagram",
            "dev",
            "web-dev",
            "web-build",
            "c-api-build",
            "cpp-build",
            "cpp-backend-build",
            "asm-build",
            "wasm-build",
            "tauri-build",
            "clean",
        };

    /// <summary>
    /// リポジトリルートのパス。リポジトリルートが見つからない場合は null。
    /// </summary>
    private readonly string? _repositoryRoot;

    /// <summary>
    /// 現在実行中のプロセス。実行中のプロセスがない場合は null。
    /// </summary>
    private Process? _runningProcess;

    /// <summary>
    /// 現在の操作の経過時間を計測する Stopwatch。操作が実行中でない場合は null。
    /// </summary>
    private Stopwatch? _stopwatch;

    /// <summary>
    /// 現在の操作の停止が要求されたかどうかを示すフラグ。
    /// </summary>
    private bool _stopRequested;

    /// <summary>
    /// MainWindow クラスの新しいインスタンスを初期化します。
    /// </summary>
    public MainWindow()
    {
        InitializeComponent();

        _repositoryRoot =
            FindRepositoryRoot(AppContext.BaseDirectory) ??
            FindRepositoryRoot(Directory.GetCurrentDirectory());

        if (_repositoryRoot is null)
        {
            RepositoryPathTextBox.Text = "Repository not found";
            OperationsPanel.IsEnabled = false;
            StatusTextBlock.Text = "Status: Repository not found";

            AppendLog(
                "[ERROR] scripts\\ops.bat が存在する" +
                "リポジトリルートを検出できませんでした。");

            return;
        }

        RepositoryPathTextBox.Text = _repositoryRoot;
        AppendLog("Whitebase Control Panel initialized.");
        AppendLog($"Repository: {_repositoryRoot}");
    }

    /// <summary>
    /// 操作ボタンがクリックされたときに呼び出されるイベントハンドラー。
    /// </summary>
    /// <param name="sender"></param>
    /// <param name="e"></param>
    private async void OperationButton_Click(
        object sender,
        RoutedEventArgs e)
    {
        if (sender is not Button { Tag: string command })
        {
            return;
        }

        await RunOperationAsync(command);
    }

    /// <summary>
    /// 指定されたコマンドを実行する非同期メソッド。
    /// </summary>
    /// <param name="command"></param>
    /// <returns></returns>
    /// <exception cref="InvalidOperationException"></exception>
    private async Task RunOperationAsync(string command)
    {
        if (_runningProcess is not null)
        {
            AppendLog("[WARN] 別の操作が実行中です。");
            return;
        }

        if (_repositoryRoot is null)
        {
            AppendLog("[ERROR] リポジトリルートが設定されていません。");
            return;
        }

        if (!AllowedCommands.Contains(command))
        {
            AppendLog($"[ERROR] 許可されていないコマンドです: {command}");
            return;
        }

        string opsPath = Path.Combine(
            _repositoryRoot,
            "scripts",
            "ops.bat");

        if (!File.Exists(opsPath))
        {
            AppendLog($"[ERROR] ops.batが見つかりません: {opsPath}");
            return;
        }

        OutputTextBox.Clear();

        AppendLog($"> scripts\\ops.bat {command}");
        AppendLog(string.Empty);

        _stopRequested = false;
        _stopwatch = Stopwatch.StartNew();

        SetRunningState(command);

        Process? process = null;

        try
        {
            var startInfo = new ProcessStartInfo
            {
                FileName =
                    Environment.GetEnvironmentVariable("ComSpec")
                    ?? "cmd.exe",

                WorkingDirectory = _repositoryRoot,

                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true,

                StandardOutputEncoding = Encoding.UTF8,
                StandardErrorEncoding = Encoding.UTF8,

                Arguments = $"/d /c \"call scripts\\ops.bat {command}\"",
            };

            startInfo.Environment["NO_COLOR"] = "1";
            startInfo.Environment["FORCE_COLOR"] = "0";
            startInfo.Environment["CLICOLOR"] = "0";
            startInfo.Environment["CARGO_TERM_COLOR"] = "never";

            process = new Process
            {
                StartInfo = startInfo,
                EnableRaisingEvents = true,
            };

            if (!process.Start())
            {
                throw new InvalidOperationException(
                    "プロセスを開始できませんでした。");
            }

            _runningProcess = process;

            Task standardOutputTask = ReadStreamAsync(
                process.StandardOutput,
                isError: false);

            Task standardErrorTask = ReadStreamAsync(
                process.StandardError,
                isError: true);

            await process.WaitForExitAsync();

            await Task.WhenAll(
                standardOutputTask,
                standardErrorTask);

            int exitCode = process.ExitCode;

            _stopwatch.Stop();

            AppendLog(string.Empty);

            if (_stopRequested)
            {
                AppendLog("[CANCELLED] Operation stopped.");
                StatusTextBlock.Text = "Status: Cancelled";
            }
            else if (exitCode == 0)
            {
                AppendLog("[SUCCESS] Operation completed.");
                StatusTextBlock.Text = "Status: Operational";
            }
            else
            {
                AppendLog(
                    $"[FAILED] Operation exited with code {exitCode}.");

                StatusTextBlock.Text = "Status: Failed";
            }

            ExitCodeTextBlock.Text = $"Exit: {exitCode}";
            ElapsedTextBlock.Text =
                $"Elapsed: {FormatElapsed(_stopwatch.Elapsed)}";
        }
        catch (Exception exception)
        {
            _stopwatch?.Stop();

            AppendLog(string.Empty);
            AppendLog($"[ERROR] {exception.Message}");

            StatusTextBlock.Text = "Status: Error";
            ExitCodeTextBlock.Text = "Exit: -";

            if (_stopwatch is not null)
            {
                ElapsedTextBlock.Text =
                    $"Elapsed: {FormatElapsed(_stopwatch.Elapsed)}";
            }
        }
        finally
        {
            if (ReferenceEquals(
                    _runningProcess,
                    process))
            {
                _runningProcess = null;
            }

            process?.Dispose();

            _stopRequested = false;

            OperationsPanel.IsEnabled =
                _repositoryRoot is not null;

            StopButton.IsEnabled = false;

            StatusTextBlock.Text =
                _repositoryRoot is null
                    ? "Status: Repository not found"
                    : "Status: Ready";
        }
    }

    /// <summary>
    /// 指定されたStreamReaderから非同期に行を読み取り、ログへ追加します。
    /// </summary>
    /// <param name="reader">読み取り対象のStreamReader。</param>
    /// <param name="isError">標準エラー出力かどうかを示す値。</param>
    /// <returns>非同期処理を表すTask。</returns>
    private async Task ReadStreamAsync(
        StreamReader reader,
        bool isError)
    {
        while (await reader.ReadLineAsync() is { } line)
        {
            string sanitizedLine = SanitizeTerminalOutput(line);

            string output = isError
                ? $"[STDERR] {sanitizedLine}"
                : sanitizedLine;

            await Dispatcher.InvokeAsync(
                () => AppendLog(output));
        }
    }

    /// <summary>
    /// ターミナル出力をサニタイズする。改行コードやベル文字を削除する。
    /// </summary>
    /// <param name="value">サニタイズする文字列。</param>
    /// <returns>サニタイズされた文字列。</returns>
    private static string SanitizeTerminalOutput(string value)
    {
        string sanitized = AnsiEscapeRegex.Replace(value, string.Empty);

        return sanitized
            .Replace("\r", string.Empty)
            .Replace("\a", string.Empty);
    }

    /// <summary>
    /// 停止ボタンがクリックされたときに呼び出されるイベントハンドラー。
    /// </summary>
    /// <param name="sender">イベントの送信者。</param>
    /// <param name="e">イベントのデータ。</param>
    /// <summary>
    /// 停止ボタンがクリックされたときに呼び出されるイベントハンドラー。
    /// </summary>
    /// <param name="sender">イベントの送信者。</param>
    /// <param name="e">イベントのデータ。</param>
    /// <summary>
    /// 停止ボタンがクリックされたときに呼び出されるイベントハンドラー。
    /// </summary>
    /// <param name="sender">イベントの送信者。</param>
    /// <param name="e">イベントのデータ。</param>
    private async void StopButton_Click(
        object sender,
        RoutedEventArgs e)
    {
        Process? process = _runningProcess;

        if (process is null)
        {
            AppendLog("[STOP] No process is running.");
            return;
        }

        _stopRequested = true;
        StopButton.IsEnabled = false;
        StatusTextBlock.Text = "Status: Stopping...";

        AppendLog(string.Empty);
        AppendLog("[STOP] Requesting graceful shutdown...");

        try
        {
            ProcessShutdownResult result =
                await ProcessShutdown.StopAsync(
                    process,
                    applicationProcessName: "whitebase-app",
                    gracefulTimeout: TimeSpan.FromSeconds(5),
                    forceTimeout: TimeSpan.FromSeconds(5));

            AppendLog(
                result.ApplicationFound
                    ? $"[STOP] Tauri process found. PID: {result.ApplicationProcessId}"
                    : "[STOP] Tauri process was not found.");

            if (result.CloseRequested)
            {
                AppendLog("[STOP] WM_CLOSE was sent.");
            }

            if (!string.IsNullOrWhiteSpace(
                    result.GracefulShutdownError))
            {
                AppendLog(
                    $"[STOP] Graceful shutdown warning: " +
                    result.GracefulShutdownError);
            }

            AppendLog(result.Status switch
            {
                ProcessShutdownStatus.AlreadyExited =>
                    "[STOP] Process had already exited.",

                ProcessShutdownStatus.ClosedGracefully =>
                    "[STOP] Development environment closed gracefully.",

                ProcessShutdownStatus.TerminatedForcefully =>
                    "[STOP] Development environment was terminated forcefully.",

                ProcessShutdownStatus.TimedOut =>
                    "[ERROR] Process tree did not stop within the timeout.",

                _ => throw new InvalidOperationException(
                    $"Unknown shutdown status: {result.Status}"),
            });

            if (result.Status == ProcessShutdownStatus.TimedOut)
            {
                StatusTextBlock.Text = "Status: Stop failed";
                StopButton.IsEnabled = true;
            }
        }
        catch (Exception exception)
        {
            AppendLog(
                $"[ERROR] Stop failed: {exception.Message}");

            StatusTextBlock.Text = "Status: Stop failed";
            StopButton.IsEnabled = true;
        }
    }

    /// <summary>
    /// ウィンドウが閉じられるときに呼び出されるイベントハンドラー。
    /// </summary>
    /// <param name="sender">イベントの送信者。</param>
    /// <param name="e">イベントのデータ。</param>
    private void Window_Closing(
        object? sender,
        System.ComponentModel.CancelEventArgs e)
    {
        Process? process = _runningProcess;

        if (process is null)
        {
            return;
        }

        try
        {
            if (!process.HasExited)
            {
                _stopRequested = true;
                process.Kill(entireProcessTree: true);
            }
        }
        catch
        {
            // ウィンドウ終了時なので、停止処理の例外は無視する。
        }
    }

    /// <summary>
    /// プロセスが実行中であることを示す状態を設定する。
    /// </summary>
    /// <param name="command">実行されるコマンド。</param>
    private void SetRunningState(string command)
    {
        OperationsPanel.IsEnabled = false;
        StopButton.IsEnabled = true;

        StatusTextBlock.Text = $"Status: Running {command}";
        ExitCodeTextBlock.Text = "Exit: -";
        ElapsedTextBlock.Text = "Elapsed: running...";
    }

    /// <summary>
    /// 指定されたメッセージをログに追加する。
    /// </summary>
    /// <param name="message">追加するメッセージ。</param>
    private void AppendLog(string message)
    {
        OutputTextBox.AppendText(
            message + Environment.NewLine);

        OutputTextBox.ScrollToEnd();
    }

    /// <summary>
    /// 指定された TimeSpan を "hh:mm:ss.fff" 形式の文字列にフォーマットする。
    /// </summary>
    /// <param name="elapsed">フォーマットする TimeSpan。</param>
    /// <returns></returns>
    private static string FormatElapsed(TimeSpan elapsed)
    {
        return elapsed.ToString(@"hh\:mm\:ss\.fff");
    }

    /// <summary>
    /// リポジトリのルートを探索する。
    /// </summary>
    /// <param name="startPath">探索を開始するパス。</param>
    /// <returns>リポジトリのルートパス、または null。</returns>
    private static string? FindRepositoryRoot(
        string startPath)
    {
        var directory = new DirectoryInfo(startPath);

        while (directory is not null)
        {
            string opsPath = Path.Combine(
                directory.FullName,
                "scripts",
                "ops.bat");

            if (File.Exists(opsPath))
            {
                return directory.FullName;
            }

            directory = directory.Parent;
        }

        return null;
    }
}