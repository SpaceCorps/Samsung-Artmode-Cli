using Spectre.Console.Cli;
using SamsungArtMode.Console.Commands;

var app = new CommandApp();

app.Configure(config =>
{
    config.SetApplicationName("samsung-art");
    config.SetApplicationVersion("1.0.0");

    config.AddCommand<PairCommand>("pair")
        .WithDescription("Pair with a Samsung Frame TV and get an auth token");

    config.AddCommand<StatusCommand>("status")
        .WithDescription("Get current art mode status");

    config.AddCommand<ListCommand>("list")
        .WithDescription("List images on the TV");

    config.AddCommand<CurrentCommand>("current")
        .WithDescription("Get the currently displayed artwork");

    config.AddCommand<SelectCommand>("select")
        .WithDescription("Select an image to display");

    config.AddCommand<UploadCommand>("upload")
        .WithDescription("Upload an image to the TV");

    config.AddCommand<DeleteCommand>("delete")
        .WithDescription("Delete an image from the TV");

    config.AddCommand<FavoriteCommand>("favorite")
        .WithDescription("Add or remove an image from favorites");

    config.AddBranch("brightness", brightness =>
    {
        brightness.SetDescription("Display brightness controls");
        brightness.AddCommand<GetBrightnessCommand>("get");
        brightness.AddCommand<SetBrightnessCommand>("set");
    });

    config.AddBranch("matte", matte =>
    {
        matte.SetDescription("Matte/frame controls");
        matte.AddCommand<ListMattesCommand>("list");
        matte.AddCommand<SetMatteCommand>("set");
    });

    config.AddCommand<SlideshowCommand>("slideshow")
        .WithDescription("Configure slideshow settings");
});

return app.Run(args);
