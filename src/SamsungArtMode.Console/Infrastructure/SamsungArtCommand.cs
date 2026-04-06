using System.Net.WebSockets;
using System.Text.Json;
using Spectre.Console.Cli;

namespace SamsungArtMode.Console.Infrastructure;

public abstract class SamsungArtCommand<TSettings> : AsyncCommand<TSettings>
    where TSettings : GlobalSettings
{
    public sealed override async Task<int> ExecuteAsync(CommandContext context, TSettings settings, CancellationToken cancellation)
    {
        try
        {
            var host = settings.ResolveHost();
            var token = settings.ResolveToken();
            using var client = new SamsungTvClient(host, token, settings.Verbose);
            await client.ConnectAsync(cancellation);
            var result = await ExecuteAsync(client, settings, cancellation);
            OutputHelper.Write(result, settings.Format);
            return 0;
        }
        catch (SamsungApiException ex)
        {
            OutputHelper.WriteError(ex.Message);
            return 1;
        }
        catch (SamsungException ex)
        {
            OutputHelper.WriteError(ex.Message);
            return 1;
        }
        catch (WebSocketException ex)
        {
            OutputHelper.WriteError($"Connection failed: {ex.Message}");
            return 2;
        }
    }

    protected abstract Task<object> ExecuteAsync(SamsungTvClient client, TSettings settings, CancellationToken ct);

    protected static object ToObject(JsonElement el)
    {
        return el.ValueKind switch
        {
            JsonValueKind.Object => el.EnumerateObject().ToDictionary(p => p.Name, p => ToObject(p.Value)),
            JsonValueKind.Array => el.EnumerateArray().Select(ToObject).ToList(),
            JsonValueKind.String => el.GetString()!,
            JsonValueKind.Number => el.TryGetInt64(out var l) ? l : el.GetDouble(),
            JsonValueKind.True => true,
            JsonValueKind.False => false,
            _ => null!
        };
    }
}
