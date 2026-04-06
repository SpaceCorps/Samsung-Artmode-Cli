using System.ComponentModel;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class FavoriteCommand : SamsungArtCommand<FavoriteCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<CONTENT_ID>")]
        [Description("Content ID of the image")]
        public required string ContentId { get; init; }

        [CommandOption("--remove")]
        [Description("Remove from favorites instead of adding")]
        [DefaultValue(false)]
        public bool Remove { get; init; }
    }

    protected override async Task<object> ExecuteAsync(SamsungTvClient client, Settings settings, CancellationToken ct)
    {
        var result = await client.SendArtRequestAsync("change_favorite", new Dictionary<string, object>
        {
            ["content_id"] = settings.ContentId,
            ["status"] = settings.Remove ? "off" : "on"
        }, ct);
        return ToObject(result);
    }
}
