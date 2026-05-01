using Producthunt.Console.Commands;
using Spectre.Console.Cli;

var app = new CommandApp();

app.Configure(config =>
{
    config.SetApplicationName("producthunt");

    config.AddCommand<SearchCommand>("search")
        .WithDescription("Search Product Hunt for products and launches");

    config.AddCommand<ScrapeCommand>("scrape")
        .WithDescription("Scrape a Product Hunt URL (product page, launch, or collection)");
});

return app.Run(args);
