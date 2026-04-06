using System.ComponentModel;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class ListMattesCommand : SamsungArtCommand<GlobalSettings>
{
    protected override async Task<object> ExecuteAsync(SamsungTvClient client, GlobalSettings settings, CancellationToken ct)
    {
        var result = await client.SendArtRequestAsync("get_matte_list", ct: ct);
        return ToObject(result);
    }
}

public sealed class SetMatteCommand : SamsungArtCommand<SetMatteCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<CONTENT_ID>")]
        [Description("Content ID of the image")]
        public required string ContentId { get; init; }

        [CommandArgument(1, "<MATTE_ID>")]
        [Description("Matte ID to apply (e.g. flexible_polar, shadowbox_black, none)")]
        public required string MatteId { get; init; }
    }

    protected override async Task<object> ExecuteAsync(SamsungTvClient client, Settings settings, CancellationToken ct)
    {
        var result = await client.SendArtRequestAsync("change_matte", new Dictionary<string, object>
        {
            ["content_id"] = settings.ContentId,
            ["matte_id"] = settings.MatteId,
            ["portrait_matte_id"] = "none"
        }, ct);
        return ToObject(result);
    }
}
