using System.ComponentModel;
using Spectre.Console;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class UploadCommand : SamsungArtCommand<UploadCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<FILE>")]
        [Description("Path to the image file (PNG or JPEG)")]
        public required string File { get; init; }

        [CommandOption("--matte <MATTE_ID>")]
        [Description("Matte to apply (e.g. none, flexible_polar, shadowbox_black)")]
        public string? Matte { get; init; }

        [CommandOption("--select")]
        [Description("Select the uploaded image as active artwork")]
        [DefaultValue(false)]
        public bool Select { get; init; }

        public override ValidationResult Validate()
        {
            if (!System.IO.File.Exists(File))
                return ValidationResult.Error($"File not found: {File}");
            return base.Validate();
        }
    }

    protected override async Task<object> ExecuteAsync(SamsungTvClient client, Settings settings, CancellationToken ct)
    {
        var contentId = await client.UploadImageAsync(settings.File, settings.Matte, ct);

        if (settings.Select)
        {
            await client.SendArtRequestAsync("select_image", new Dictionary<string, object>
            {
                ["content_id"] = contentId,
                ["show"] = true
            }, ct);
        }

        return new { content_id = contentId, selected = settings.Select };
    }
}
