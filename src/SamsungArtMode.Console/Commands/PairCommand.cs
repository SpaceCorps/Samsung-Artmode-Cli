using System.ComponentModel;
using System.Net.WebSockets;
using Spectre.Console;
using Spectre.Console.Cli;
using SamsungArtMode.Console.Infrastructure;

namespace SamsungArtMode.Console.Commands;

public sealed class PairCommand : AsyncCommand<PairCommand.Settings>
{
    public sealed class Settings : CommandSettings
    {
        [CommandArgument(0, "[HOST]")]
        [Description("TV IP address or hostname. If omitted, auto-discovers on the local network.")]
        public string? Host { get; init; }
    }

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        try
        {
            var host = settings.Host;

            if (host is null)
            {
                host = await DiscoverHostAsync(cancellation);
                if (host is null) return 1;
            }

            AnsiConsole.MarkupLine("[yellow]Connecting to TV... Check your TV and press Allow when prompted.[/]");

            using var client = new SamsungTvClient(host, token: null, verbose: false);
            var token = await client.ConnectAsync(cancellation);

            if (token is null)
            {
                AnsiConsole.MarkupLine("[red]No token received. The TV may have denied the connection.[/]");
                return 1;
            }

            AnsiConsole.MarkupLine("[green]Paired successfully![/]");
            AnsiConsole.WriteLine();
            AnsiConsole.MarkupLine($"Token: [bold]{token}[/]");
            AnsiConsole.WriteLine();
            AnsiConsole.MarkupLine("Save it as an environment variable:");
            AnsiConsole.MarkupLine($"  [dim]set SAMSUNG_TV_HOST={host}[/]");
            AnsiConsole.MarkupLine($"  [dim]set SAMSUNG_TV_TOKEN={token}[/]");
            return 0;
        }
        catch (WebSocketException ex)
        {
            OutputHelper.WriteError($"Connection failed: {ex.Message}");
            return 2;
        }
    }

    private static async Task<string?> DiscoverHostAsync(CancellationToken ct)
    {
        AnsiConsole.MarkupLine("[yellow]Scanning network for Samsung TVs...[/]");

        var tvs = await SamsungDiscovery.DiscoverAsync(TimeSpan.FromSeconds(5), ct);

        if (tvs.Count == 0)
        {
            AnsiConsole.MarkupLine("[red]No Samsung TVs found on the network.[/]");
            AnsiConsole.MarkupLine("[dim]Make sure the TV is on and connected to the same network.[/]");
            AnsiConsole.MarkupLine("[dim]You can also specify the IP directly: samsung-art pair 192.168.1.xxx[/]");
            return null;
        }

        if (tvs.Count == 1)
        {
            var tv = tvs[0];
            AnsiConsole.MarkupLine($"[green]Found:[/] {tv.Host} {(tv.Name is not null ? $"({Markup.Escape(tv.Name)})" : "")}");
            return tv.Host;
        }

        // Multiple TVs found — let the user pick
        var selected = AnsiConsole.Prompt(
            new SelectionPrompt<SamsungDiscovery.DiscoveredTv>()
                .Title("Multiple Samsung TVs found. Which one?")
                .UseConverter(tv => $"{tv.Host}{(tv.Name is not null ? $" ({tv.Name})" : "")}")
                .AddChoices(tvs));

        return selected.Host;
    }
}
