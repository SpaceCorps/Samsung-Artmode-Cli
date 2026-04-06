using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class CurrentCommand : SamsungArtCommand<GlobalSettings>
{
    protected override async Task<object> ExecuteAsync(SamsungTvClient client, GlobalSettings settings, CancellationToken ct)
    {
        var result = await client.SendArtRequestAsync("get_current_artwork", ct: ct);
        return ToObject(result);
    }
}
