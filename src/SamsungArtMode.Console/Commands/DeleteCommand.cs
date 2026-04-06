using System.ComponentModel;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class DeleteCommand : SamsungArtCommand<DeleteCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<CONTENT_ID>")]
        [Description("Content ID of the image to delete")]
        public required string ContentId { get; init; }
    }

    protected override async Task<object> ExecuteAsync(SamsungTvClient client, Settings settings, CancellationToken ct)
    {
        var result = await client.SendArtRequestAsync("delete_image_list", new Dictionary<string, object>
        {
            ["content_id_list"] = new[] { new { content_id = settings.ContentId } }
        }, ct);
        return ToObject(result);
    }
}
