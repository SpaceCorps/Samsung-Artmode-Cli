using System.Text.Json;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace SamsungArtMode.Console.Infrastructure;

public static class OutputHelper
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull
    };

    private static readonly ISerializer YamlSerializer = new SerializerBuilder()
        .WithNamingConvention(CamelCaseNamingConvention.Instance)
        .ConfigureDefaultValuesHandling(DefaultValuesHandling.OmitNull)
        .Build();

    public static void Write(object data, string format)
    {
        var output = format.ToLowerInvariant() switch
        {
            "json" => JsonSerializer.Serialize(data, JsonOptions),
            _ => YamlSerializer.Serialize(data).TrimEnd()
        };
        System.Console.WriteLine(output);
    }

    public static void WriteError(string message)
    {
        System.Console.Error.WriteLine(YamlSerializer.Serialize(new { error = message }).TrimEnd());
    }
}
