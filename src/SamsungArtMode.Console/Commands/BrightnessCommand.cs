using System.ComponentModel;
using Spectre.Console;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class GetBrightnessCommand : SamsungArtCommand<GlobalSettings>
{
    protected override async Task<object> ExecuteAsync(SamsungTvClient client, GlobalSettings settings, CancellationToken ct)
    {
        var result = await client.SendArtRequestAsync("get_brightness", ct: ct);
        return ToObject(result);
    }
}

public sealed class SetBrightnessCommand : SamsungArtCommand<SetBrightnessCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<VALUE>")]
        [Description("Brightness value (0-10)")]
        public int Value { get; init; }

        public override ValidationResult Validate()
        {
            if (Value is < 0 or > 10)
                return ValidationResult.Error("Brightness must be between 0 and 10.");
            return base.Validate();
        }
    }

    protected override async Task<object> ExecuteAsync(SamsungTvClient client, Settings settings, CancellationToken ct)
    {
        var result = await client.SendArtRequestAsync("set_brightness", new Dictionary<string, object>
        {
            ["value"] = settings.Value
        }, ct);
        return ToObject(result);
    }
}
