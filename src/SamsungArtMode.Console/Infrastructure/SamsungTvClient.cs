using System.Net.Security;
using System.Net.Sockets;
using System.Net.WebSockets;
using System.Security.Cryptography.X509Certificates;
using System.Text;
using System.Text.Json;

namespace SamsungArtMode.Console.Infrastructure;

public sealed class SamsungTvClient : IDisposable
{
    private const string ArtEndpoint = "com.samsung.art-app";
    private readonly string _host;
    private readonly string? _token;
    private readonly bool _verbose;
    private ClientWebSocket? _ws;

    public SamsungTvClient(string host, string? token = null, bool verbose = false)
    {
        _host = host;
        _token = token;
        _verbose = verbose;
    }

    public async Task<string?> ConnectAsync(CancellationToken ct = default)
    {
        _ws = new ClientWebSocket();
        _ws.Options.RemoteCertificateValidationCallback = (_, _, _, _) => true;

        var name = Convert.ToBase64String(Encoding.UTF8.GetBytes("SamsungArtCli"));
        var uri = _token is not null
            ? new Uri($"wss://{_host}:8002/api/v2/channels/{ArtEndpoint}?name={name}&token={_token}")
            : new Uri($"ws://{_host}:8001/api/v2/channels/{ArtEndpoint}?name={name}");

        if (_verbose)
            System.Console.Error.WriteLine($"WS connecting to {uri.Scheme}://{uri.Host}:{uri.Port}...");

        await _ws.ConnectAsync(uri, ct);

        // Read the connect response to get the token
        var response = await ReceiveRawAsync(ct);
        if (_verbose)
            System.Console.Error.WriteLine($"WS << {response}");

        using var doc = JsonDocument.Parse(response);
        var root = doc.RootElement;

        if (root.TryGetProperty("data", out var data) && data.TryGetProperty("token", out var tokenEl))
            return tokenEl.GetString();

        return null;
    }

    public async Task<JsonElement> SendArtRequestAsync(string request, Dictionary<string, object>? extra = null, CancellationToken ct = default)
    {
        EnsureConnected();

        var payload = new Dictionary<string, object> { ["request"] = request, ["id"] = Guid.NewGuid().ToString() };
        if (extra is not null)
            foreach (var kv in extra)
                payload[kv.Key] = kv.Value;

        var dataJson = JsonSerializer.Serialize(payload);
        var envelope = JsonSerializer.Serialize(new
        {
            method = "ms.channel.emit",
            @params = new
            {
                @event = "art_app_request",
                to = "host",
                data = dataJson
            }
        });

        if (_verbose)
            System.Console.Error.WriteLine($"WS >> {envelope}");

        var bytes = Encoding.UTF8.GetBytes(envelope);
        await _ws!.SendAsync(bytes, WebSocketMessageType.Text, true, ct);

        // Wait for the response matching our request
        return await WaitForResponseAsync(request, ct);
    }

    public async Task<string> UploadImageAsync(string filePath, string? matteId = null, CancellationToken ct = default)
    {
        EnsureConnected();

        var fileBytes = await File.ReadAllBytesAsync(filePath, ct);
        var ext = Path.GetExtension(filePath).TrimStart('.').ToLowerInvariant();
        if (ext is "jpg") ext = "jpeg";
        if (ext is not ("png" or "jpeg"))
            throw new SamsungException($"Unsupported image format: {ext}. Use PNG or JPEG.");

        var requestId = Guid.NewGuid().ToString();
        var connId = Random.Shared.Next(1, 999999);

        var sendPayload = new Dictionary<string, object>
        {
            ["request"] = "send_image",
            ["file_type"] = ext,
            ["file_size"] = fileBytes.Length,
            ["id"] = Guid.NewGuid().ToString(),
            ["request_id"] = requestId,
            ["matte_id"] = matteId ?? "none",
            ["portrait_matte_id"] = "none",
            ["image_date"] = DateTime.Now.ToString("yyyy:MM:dd HH:mm:ss"),
            ["conn_info"] = new Dictionary<string, object>
            {
                ["d2d_mode"] = "socket",
                ["connection_id"] = connId,
                ["id"] = Guid.NewGuid().ToString()
            }
        };

        var dataJson = JsonSerializer.Serialize(sendPayload);
        var envelope = JsonSerializer.Serialize(new
        {
            method = "ms.channel.emit",
            @params = new
            {
                @event = "art_app_request",
                to = "host",
                data = dataJson
            }
        });

        if (_verbose)
            System.Console.Error.WriteLine($"WS >> {envelope}");

        await _ws!.SendAsync(Encoding.UTF8.GetBytes(envelope), WebSocketMessageType.Text, true, ct);

        // Wait for ready_to_use with connection info
        var connInfo = await WaitForUploadReadyAsync(ct);
        var ip = connInfo.GetProperty("ip").GetString()!;
        var port = connInfo.GetProperty("port").GetInt32();
        var secured = connInfo.TryGetProperty("secured", out var sec) && sec.GetBoolean();
        var secKey = connInfo.TryGetProperty("key", out var key) ? key.GetString() : connInfo.TryGetProperty("secKey", out var sk) ? sk.GetString() : "";

        if (_verbose)
            System.Console.Error.WriteLine($"TCP uploading to {ip}:{port} (secured={secured}, size={fileBytes.Length})");

        // Phase 2: TCP binary upload
        await UploadViaTcpAsync(ip, port, secured, secKey ?? "", ext, fileBytes, ct);

        // Wait for image_added event
        var contentId = await WaitForImageAddedAsync(ct);
        return contentId;
    }

    private async Task UploadViaTcpAsync(string ip, int port, bool secured, string secKey, string fileType, byte[] fileBytes, CancellationToken ct)
    {
        using var tcp = new TcpClient();
        await tcp.ConnectAsync(ip, port, ct);

        Stream stream = tcp.GetStream();
        if (secured)
        {
            var ssl = new SslStream(stream, false, (_, _, _, _) => true);
            await ssl.AuthenticateAsClientAsync(new SslClientAuthenticationOptions
            {
                TargetHost = ip,
                RemoteCertificateValidationCallback = (_, _, _, _) => true
            }, ct);
            stream = ssl;
        }

        var header = JsonSerializer.Serialize(new
        {
            num = 0,
            total = 1,
            fileLength = fileBytes.Length,
            fileName = "upload",
            fileType,
            secKey,
            version = "0.0.1"
        });

        var headerBytes = Encoding.ASCII.GetBytes(header);
        var lenBytes = BitConverter.GetBytes(System.Net.IPAddress.HostToNetworkOrder(headerBytes.Length));

        await stream.WriteAsync(lenBytes, ct);
        await stream.WriteAsync(headerBytes, ct);

        // Send in 64KB chunks
        const int chunkSize = 65536;
        for (var offset = 0; offset < fileBytes.Length; offset += chunkSize)
        {
            var count = Math.Min(chunkSize, fileBytes.Length - offset);
            await stream.WriteAsync(fileBytes.AsMemory(offset, count), ct);
        }

        await stream.FlushAsync(ct);
    }

    private async Task<JsonElement> WaitForResponseAsync(string request, CancellationToken ct)
    {
        var timeout = CancellationTokenSource.CreateLinkedTokenSource(ct);
        timeout.CancelAfter(TimeSpan.FromSeconds(15));

        while (!timeout.Token.IsCancellationRequested)
        {
            var raw = await ReceiveRawAsync(timeout.Token);
            if (_verbose)
                System.Console.Error.WriteLine($"WS << {raw}");

            using var doc = JsonDocument.Parse(raw);
            var root = doc.RootElement;

            if (!root.TryGetProperty("data", out var dataStr)) continue;

            // data might be a string (double-serialized) or object
            JsonElement data;
            if (dataStr.ValueKind == JsonValueKind.String)
            {
                using var inner = JsonDocument.Parse(dataStr.GetString()!);
                data = inner.RootElement.Clone();
            }
            else
            {
                data = dataStr.Clone();
            }

            if (data.TryGetProperty("request", out var req) && req.GetString() == request)
                return data;

            // Also check event field
            if (data.TryGetProperty("event", out var evt) && evt.GetString() == request)
                return data;
        }

        throw new SamsungApiException("Timed out waiting for response", request);
    }

    private async Task<JsonElement> WaitForUploadReadyAsync(CancellationToken ct)
    {
        var timeout = CancellationTokenSource.CreateLinkedTokenSource(ct);
        timeout.CancelAfter(TimeSpan.FromSeconds(30));

        while (!timeout.Token.IsCancellationRequested)
        {
            var raw = await ReceiveRawAsync(timeout.Token);
            if (_verbose)
                System.Console.Error.WriteLine($"WS << {raw}");

            using var doc = JsonDocument.Parse(raw);
            var data = ExtractData(doc.RootElement);
            if (data is null) continue;

            if (data.Value.TryGetProperty("event", out var evt) && evt.GetString() == "ready_to_use")
            {
                if (data.Value.TryGetProperty("conn_info", out var ci))
                    return ci.Clone();
            }

            // Some TVs return it nested differently
            if (data.Value.TryGetProperty("request", out var req) && req.GetString() == "ready_to_use")
            {
                if (data.Value.TryGetProperty("conn_info", out var ci2))
                    return ci2.Clone();
            }
        }

        throw new SamsungApiException("Timed out waiting for upload ready signal", "send_image");
    }

    private async Task<string> WaitForImageAddedAsync(CancellationToken ct)
    {
        var timeout = CancellationTokenSource.CreateLinkedTokenSource(ct);
        timeout.CancelAfter(TimeSpan.FromSeconds(30));

        while (!timeout.Token.IsCancellationRequested)
        {
            var raw = await ReceiveRawAsync(timeout.Token);
            if (_verbose)
                System.Console.Error.WriteLine($"WS << {raw}");

            using var doc = JsonDocument.Parse(raw);
            var data = ExtractData(doc.RootElement);
            if (data is null) continue;

            if (data.Value.TryGetProperty("event", out var evt) && evt.GetString() == "image_added")
            {
                if (data.Value.TryGetProperty("content_id", out var cid))
                    return cid.GetString()!;
            }

            if (data.Value.TryGetProperty("request", out var req) && req.GetString() == "image_added")
            {
                if (data.Value.TryGetProperty("content_id", out var cid2))
                    return cid2.GetString()!;
            }
        }

        throw new SamsungApiException("Timed out waiting for image upload confirmation", "send_image");
    }

    private static JsonElement? ExtractData(JsonElement root)
    {
        if (!root.TryGetProperty("data", out var dataStr)) return null;
        if (dataStr.ValueKind == JsonValueKind.String)
        {
            using var inner = JsonDocument.Parse(dataStr.GetString()!);
            return inner.RootElement.Clone();
        }
        return dataStr;
    }

    private async Task<string> ReceiveRawAsync(CancellationToken ct)
    {
        EnsureConnected();
        var buffer = new byte[65536];
        using var ms = new MemoryStream();

        WebSocketReceiveResult result;
        do
        {
            result = await _ws!.ReceiveAsync(buffer, ct);
            ms.Write(buffer, 0, result.Count);
        } while (!result.EndOfMessage);

        return Encoding.UTF8.GetString(ms.ToArray());
    }

    private void EnsureConnected()
    {
        if (_ws is null || _ws.State != WebSocketState.Open)
            throw new SamsungException("Not connected to TV. Call ConnectAsync first.");
    }

    public void Dispose()
    {
        if (_ws is { State: WebSocketState.Open })
        {
            try { _ws.CloseAsync(WebSocketCloseStatus.NormalClosure, null, CancellationToken.None).GetAwaiter().GetResult(); }
            catch { /* best effort */ }
        }
        _ws?.Dispose();
    }
}
