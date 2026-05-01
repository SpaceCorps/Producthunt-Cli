using System.ComponentModel;
using Producthunt.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Producthunt.Console.Commands;

public sealed class SearchCommand : AsyncCommand<SearchCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<QUERY>")]
        [Description("Search query (product name, category, or keyword)")]
        public required string Query { get; init; }

        [CommandOption("--max <N>")]
        [Description("Maximum results to return")]
        [DefaultValue(10)]
        public int Max { get; init; } = 10;

        [CommandOption("--sort <SORT>")]
        [Description("Sort by: relevance, newest, popular")]
        [DefaultValue("relevance")]
        public string Sort { get; init; } = "relevance";
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        AnsiConsole.MarkupLine("[grey]Searching Product Hunt (this may take 30–60s)...[/]");

        using var client = settings.CreateClient();
        var doc = await client.SearchAsync(new SearchInput
        {
            SearchTerms = [settings.Query],
            MaxResults = settings.Max,
            Sort = settings.Sort
        });

        YamlOutput.Write(doc);
        return 0;
    }
}
