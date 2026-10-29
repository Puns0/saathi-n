using System;
using System.IO;
using System.Net;
using System.Threading;
using System.Diagnostics;
using System.Collections.Generic;

namespace VoidRelayLauncher
{
    class Program
    {
        private static HttpListener _listener;
        private static string _distPath;
        private static bool _running = true;

        private static readonly Dictionary<string, string> MimeTypes = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
        {
            { ".html", "text/html; charset=utf-8" },
            { ".js", "application/javascript; charset=utf-8" },
            { ".css", "text/css; charset=utf-8" },
            { ".json", "application/json" },
            { ".svg", "image/svg+xml" },
            { ".png", "image/png" },
            { ".jpg", "image/jpeg" },
            { ".jpeg", "image/jpeg" },
            { ".ico", "image/x-icon" },
            { ".woff", "font/woff" },
            { ".woff2", "font/woff2" },
            { ".ttf", "font/ttf" },
            { ".wasm", "application/wasm" }
        };

        static void Main(string[] args)
        {
            try
            {
                string baseDir = AppDomain.CurrentDomain.BaseDirectory;

                // 1. Locate dist directory
                string[] distCandidates = new string[]
                {
                    Path.Combine(baseDir, @"saathi\void-relay\dist"),
                    Path.Combine(baseDir, @"void-relay\dist"),
                    Path.Combine(baseDir, @"dist"),
                    Path.Combine(baseDir, @"..\dist"),
                    Path.Combine(baseDir, @"..\..\dist"),
                    Path.Combine(baseDir, @"..\..\..\dist"),
                    Path.Combine(baseDir, @"..\..\..\..\saathi\void-relay\dist"),
                    @"C:\Users\Puns0\Documents\SAATHI\saathi\void-relay\dist"
                };

                foreach (var c in distCandidates)
                {
                    if (Directory.Exists(c) && File.Exists(Path.Combine(c, "index.html")))
                    {
                        _distPath = Path.GetFullPath(c);
                        break;
                    }
                }

                // 2. Locate void-relay-core.exe
                string[] coreCandidates = new string[]
                {
                    Path.Combine(baseDir, "void-relay-core.exe"),
                    Path.Combine(baseDir, @"saathi\void-relay\src-tauri\target\release\void-relay-core.exe"),
                    Path.Combine(baseDir, @"void-relay\src-tauri\target\release\void-relay-core.exe"),
                    Path.Combine(baseDir, @"src-tauri\target\release\void-relay-core.exe"),
                    @"C:\Users\Puns0\Documents\SAATHI\saathi\void-relay\src-tauri\target\release\void-relay-core.exe"
                };

                string coreExe = null;
                foreach (var c in coreCandidates)
                {
                    if (File.Exists(c))
                    {
                        coreExe = Path.GetFullPath(c);
                        break;
                    }
                }

                if (coreExe == null)
                {
                    // Fallback to void-relay.exe if not this binary
                    string self = Process.GetCurrentProcess().MainModule.FileName;
                    string alt = @"C:\Users\Puns0\Documents\SAATHI\saathi\void-relay\src-tauri\target\release\void-relay.exe";
                    if (File.Exists(alt) && !string.Equals(self, alt, StringComparison.OrdinalIgnoreCase))
                    {
                        coreExe = alt;
                    }
                }

                if (coreExe == null)
                {
                    System.Windows.Forms.MessageBox.Show("Could not locate void-relay backend executable.", "VoidRelay", System.Windows.Forms.MessageBoxButtons.OK, System.Windows.Forms.MessageBoxIcon.Error);
                    return;
                }

                // 3. Start static server on port 1420
                if (!string.IsNullOrEmpty(_distPath))
                {
                    StartServer();
                }

                // 4. Launch void-relay-core
                ProcessStartInfo psi = new ProcessStartInfo();
                psi.FileName = coreExe;
                psi.WorkingDirectory = Path.GetDirectoryName(coreExe);
                psi.UseShellExecute = false;

                // Pass through any arguments
                if (args != null && args.Length > 0)
                {
                    psi.Arguments = string.Join(" ", args);
                }

                using (Process proc = Process.Start(psi))
                {
                    if (proc != null)
                    {
                        proc.WaitForExit();
                    }
                }
            }
            catch (Exception ex)
            {
                System.Windows.Forms.MessageBox.Show("Error starting VoidRelay: " + ex.Message, "VoidRelay Error", System.Windows.Forms.MessageBoxButtons.OK, System.Windows.Forms.MessageBoxIcon.Error);
            }
            finally
            {
                StopServer();
            }
        }

        private static void StartServer()
        {
            try
            {
                _listener = new HttpListener();
                _listener.Prefixes.Add("http://127.0.0.1:1420/");
                _listener.Prefixes.Add("http://localhost:1420/");
                _listener.Start();

                Thread listenerThread = new Thread(ListenLoop);
                listenerThread.IsBackground = true;
                listenerThread.Start();
            }
            catch
            {
                // Port might already be listening (e.g. dev server running) - ignore
            }
        }

        private static void StopServer()
        {
            _running = false;
            try
            {
                if (_listener != null && _listener.IsListening)
                {
                    _listener.Stop();
                    _listener.Close();
                }
            }
            catch { }
        }

        private static void ListenLoop()
        {
            while (_running && _listener != null && _listener.IsListening)
            {
                try
                {
                    HttpListenerContext context = _listener.GetContext();
                    ThreadPool.QueueUserWorkItem(ProcessRequest, context);
                }
                catch
                {
                    break;
                }
            }
        }

        private static void ProcessRequest(object state)
        {
            HttpListenerContext context = (HttpListenerContext)state;
            try
            {
                string rawUrl = context.Request.RawUrl;
                int qIdx = rawUrl.IndexOf('?');
                string pathPart = qIdx >= 0 ? rawUrl.Substring(0, qIdx) : rawUrl;
                pathPart = pathPart.TrimStart('/').Replace('/', Path.DirectorySeparatorChar);

                string filePath = Path.Combine(_distPath, pathPart);

                // Fallback to index.html for SPA routes or directories
                if (string.IsNullOrEmpty(pathPart) || !File.Exists(filePath))
                {
                    filePath = Path.Combine(_distPath, "index.html");
                }

                byte[] fileBytes;
                if (File.Exists(filePath))
                {
                    fileBytes = File.ReadAllBytes(filePath);
                    string ext = Path.GetExtension(filePath);
                    string contentType;
                    if (!MimeTypes.TryGetValue(ext, out contentType))
                    {
                        contentType = "application/octet-stream";
                    }

                    context.Response.ContentType = contentType;
                    context.Response.ContentLength64 = fileBytes.Length;
                    context.Response.StatusCode = 200;
                    context.Response.OutputStream.Write(fileBytes, 0, fileBytes.Length);
                }
                else
                {
                    context.Response.StatusCode = 404;
                }
            }
            catch
            {
                try { context.Response.StatusCode = 500; } catch { }
            }
            finally
            {
                try { context.Response.OutputStream.Close(); } catch { }
            }
        }
    }
}
