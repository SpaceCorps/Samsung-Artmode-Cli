using System.ComponentModel;
using Spectre.Console;
using Spectre.Console.Cli;

namespace SamsungArtMode.Console.Infrastructure;

public class GlobalSettings : CommandSettings
{
    [CommandOption("--host <HOST>")]
    [Description("TV IP address or hostname. Falls back to SAMSUNG_TV_HOST env var.")]
    public string? Host { get; init; }

    [CommandOption("--token <TOKEN>")]
    [Description("Auth token from pairing. Falls back to SAMSUNG_TV_TOKEN env var.")]
    public string? Token { get; init; }

    [CommandOption("--format <FORMAT>")]
    [Description("Output format: yaml, json (default: yaml)")]
    [DefaultValue("yaml")]
    public string Format { get; init; } = "yaml";

    [CommandOption("--verbose")]
    [Description("Print WebSocket messages to stderr")]
    [DefaultValue(false)]
    public bool Verbose { get; init; }

    public string ResolveHost() =>
        Host
        ?? Environment.GetEnvironmentVariable("SAMSUNG_TV_HOST")
        ?? throw new SamsungException("TV host required. Use --host or set SAMSUNG_TV_HOST.");

    public string? ResolveToken() =>
        Token ?? Environment.GetEnvironmentVariable("SAMSUNG_TV_TOKEN");

    public override ValidationResult Validate()
    {
        if (Format is not ("yaml" or "json"))
            return ValidationResult.Error($"Invalid format '{Format}'. Use yaml or json.");
        return ValidationResult.Success();
    }
}
