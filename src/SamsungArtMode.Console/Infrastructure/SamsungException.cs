namespace SamsungArtMode.Console.Infrastructure;

public class SamsungException(string message) : Exception(message);

public class SamsungApiException(string message, string? request = null) : SamsungException(message)
{
    public string? Request { get; } = request;
}
