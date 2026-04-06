using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Text.RegularExpressions;

namespace SamsungArtMode.Console.Infrastructure;

public static partial class SamsungDiscovery
{
    private static readonly IPAddress MulticastAddress = IPAddress.Parse("239.255.255.250");
    private const int SsdpPort = 1900;

    private const string MSearchMessage =
        "M-SEARCH * HTTP/1.1\r\n" +
        "HOST: 239.255.255.250:1900\r\n" +
        "MAN: \"ssdp:discover\"\r\n" +
        "MX: 3\r\n" +
        "ST: urn:samsung.com:device:RemoteControlReceiver:1\r\n" +
        "\r\n";

    public record DiscoveredTv(string Host, string? Name, string? Model);

    public static async Task<List<DiscoveredTv>> DiscoverAsync(TimeSpan? timeout = null, CancellationToken ct = default)
    {
        timeout ??= TimeSpan.FromSeconds(5);
        var results = new Dictionary<string, DiscoveredTv>();

        using var udp = new UdpClient();
        udp.Client.SetSocketOption(SocketOptionLevel.Socket, SocketOptionName.ReuseAddress, true);
        udp.Client.Bind(new IPEndPoint(IPAddress.Any, 0));

        var message = Encoding.UTF8.GetBytes(MSearchMessage);
        await udp.SendAsync(message, new IPEndPoint(MulticastAddress, SsdpPort), ct);

        // Also try the generic ssdp:all as fallback
        var altMessage = Encoding.UTF8.GetBytes(
            "M-SEARCH * HTTP/1.1\r\n" +
            "HOST: 239.255.255.250:1900\r\n" +
            "MAN: \"ssdp:discover\"\r\n" +
            "MX: 3\r\n" +
            "ST: ssdp:all\r\n" +
            "\r\n");
        await udp.SendAsync(altMessage, new IPEndPoint(MulticastAddress, SsdpPort), ct);

        using var cts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        cts.CancelAfter(timeout.Value);

        while (!cts.Token.IsCancellationRequested)
        {
            try
            {
                var result = await udp.ReceiveAsync(cts.Token);
                var response = Encoding.UTF8.GetString(result.Buffer);

                if (!IsSamsungTv(response)) continue;

                var host = result.RemoteEndPoint.Address.ToString();
                if (results.ContainsKey(host)) continue;

                var name = ExtractHeader(response, "SERVER") ?? ExtractHeader(response, "USN");
                var model = ExtractModelFromLocation(response);
                results[host] = new DiscoveredTv(host, name, model);
            }
            catch (OperationCanceledException)
            {
                break;
            }
        }

        return [.. results.Values];
    }

    private static bool IsSamsungTv(string response)
    {
        return response.Contains("samsung", StringComparison.OrdinalIgnoreCase)
            || response.Contains("SEC_", StringComparison.OrdinalIgnoreCase)
            || response.Contains("RemoteControlReceiver", StringComparison.OrdinalIgnoreCase);
    }

    private static string? ExtractHeader(string response, string header)
    {
        var match = Regex.Match(response, $@"(?i){header}\s*:\s*(.+?)(?:\r?\n|$)");
        return match.Success ? match.Groups[1].Value.Trim() : null;
    }

    private static string? ExtractModelFromLocation(string response)
    {
        var location = ExtractHeader(response, "LOCATION");
        if (location is null) return null;

        // Location typically contains the model info in the path
        var match = ModelPattern().Match(location);
        return match.Success ? match.Groups[1].Value : null;
    }

    [GeneratedRegex(@"/(\w+)/[\w.]+\.xml", RegexOptions.IgnoreCase)]
    private static partial Regex ModelPattern();
}
