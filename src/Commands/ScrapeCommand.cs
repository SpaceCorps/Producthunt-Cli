using System.ComponentModel;
using Producthunt.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Producthunt.Console.Commands;

public sealed class ScrapeCommand : AsyncCommand<ScrapeCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<URL>")]
        [Description("Product Hunt URL to scrape (product page, launch, or collection)")]
        public required string Url { get; init; }

        [CommandOption("--max <N>")]
        [Description("Maximum results to return")]
        [DefaultValue(10)]
        public int Max { get; init; } = 10;
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        AnsiConsole.MarkupLine("[grey]Scraping Product Hunt (this may take 30–60s)...[/]");

        using var client = settings.CreateClient();
        var doc = await client.ScrapeAsync(new ScrapeInput
        {
            DateUrl = settings.Url,
            MaxResults = settings.Max
        });

        YamlOutput.Write(doc);
        return 0;
    }
}
