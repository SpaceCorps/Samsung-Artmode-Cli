using System.ComponentModel;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class SelectCommand : SamsungArtCommand<SelectCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<CONTENT_ID>")]
        [Description("Content ID of the image to display (e.g. SAM-F0206)")]
        public required string ContentId { get; init; }

        [CommandOption("--show")]
        [Description("Immediately display the image even if TV is not in art mode")]
        [DefaultValue(true)]
        public bool Show { get; init; } = true;
    }

    protected override async Task<object> ExecuteAsync(SamsungTvClient client, Settings settings, CancellationToken ct)
    {
        var result = await client.SendArtRequestAsync("select_image", new Dictionary<string, object>
        {
            ["content_id"] = settings.ContentId,
            ["show"] = settings.Show
        }, ct);
        return ToObject(result);
    }
}
