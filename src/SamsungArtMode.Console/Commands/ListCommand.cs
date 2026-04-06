using System.ComponentModel;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class ListCommand : SamsungArtCommand<ListCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandOption("--category <CATEGORY>")]
        [Description("Filter by category (MY-C0002=My Photos, MY-C0004=Favorites, MY-C0008=Store)")]
        public string? Category { get; init; }
    }

    protected override async Task<object> ExecuteAsync(SamsungTvClient client, Settings settings, CancellationToken ct)
    {
        var extra = new Dictionary<string, object>();
        if (settings.Category is not null)
            extra["category"] = settings.Category;

        var result = await client.SendArtRequestAsync("get_content_list", extra, ct);
        return ToObject(result);
    }
}
