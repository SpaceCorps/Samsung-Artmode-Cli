using System.ComponentModel;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class SlideshowCommand : SamsungArtCommand<SlideshowCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandOption("--off")]
        [Description("Disable slideshow")]
        [DefaultValue(false)]
        public bool Off { get; init; }

        [CommandOption("--interval <MINUTES>")]
        [Description("Slideshow interval in minutes (e.g. 1, 5, 10, 30, 60)")]
        public int? Interval { get; init; }

        [CommandOption("--category <CATEGORY>")]
        [Description("Category to slideshow (MY-C0002=My Photos, MY-C0004=Favorites)")]
        [DefaultValue("MY-C0002")]
        public string Category { get; init; } = "MY-C0002";

        [CommandOption("--shuffle")]
        [Description("Shuffle the slideshow order")]
        [DefaultValue(false)]
        public bool Shuffle { get; init; }
    }

    protected override async Task<object> ExecuteAsync(SamsungTvClient client, Settings settings, CancellationToken ct)
    {
        if (settings.Off)
        {
            var result = await client.SendArtRequestAsync("set_slideshow_status", new Dictionary<string, object>
            {
                ["value"] = "off"
            }, ct);
            return ToObject(result);
        }

        var interval = settings.Interval ?? 10;
        var extra = new Dictionary<string, object>
        {
            ["value"] = interval.ToString(),
            ["category_id"] = settings.Category,
            ["type"] = settings.Shuffle ? "shuffleslideshow" : "slideshow"
        };

        var res = await client.SendArtRequestAsync("set_slideshow_status", extra, ct);
        return ToObject(res);
    }
}
