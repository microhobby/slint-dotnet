using Slint = SlintDotnet.SlintDotnet;

Console.WriteLine("Hello, World!");

Console.WriteLine("Creating...");
Slint.Create("./ui/appwindow.slint");
Console.WriteLine("Created");

Console.WriteLine("Getting properties...");
var props = Slint.GetProperties();
Console.WriteLine("Geted");

foreach (var prop in props)
{
    Console.WriteLine($"Name: {prop.typeName}");
    Console.WriteLine($"Type: {prop.typeType}");
    Console.WriteLine($"Val: {prop.typeValue}");
}

Slint.SetProperty(new Slint.DotNetValue
{
    typeName = "img",
    typeType = 3,
    typeValue = "./ui/assets/torizon_logo_white.svg",
    isStruct = false,
    structProps = new List<Slint.DotNetValue>()
});

var binding = Slint.GetProperty("Width");
Console.WriteLine($"Binding: {binding.typeName} = {binding.typeValue}");

Slint.CallCallback("printo");

Slint.SetCallback("request-increase-value", () =>
{
    var dt = Slint.GetProperty("counter");
    var sT = dt.typeValue
                .Replace("Value::Number(", "")
                .Replace(")", "");
    var val = float.Parse(sT) + 1;

    // struct
    var strut = Slint.GetStruct("testStruct");
    foreach (var fi in strut.structProps) {
        Console.WriteLine($"Field {fi.typeName} = {fi.typeValue}");
    }

    Slint.SetStruct(new Slint.DotNetValue
    {
        typeName = "testStruct",
        typeType = 4,
        isStruct = true,
        typeValue = "",
        structProps = new List<Slint.DotNetValue>
        {
            new Slint.DotNetValue
            {
                typeName = "T1",
                typeType = 0,
                typeValue = "modified",
                isStruct = false,
                structProps = new List<Slint.DotNetValue>()
            }
        }
    });

    strut = Slint.GetStruct("testStruct");
    foreach (var fi in strut.structProps) {
        Console.WriteLine($"Field {fi.typeName} = {fi.typeValue}");
    }

    Slint.SetProperty(new Slint.DotNetValue
    {
        typeName = "counter",
        typeType = 1,
        typeValue = val.ToString(),
        isStruct = false,
        structProps = new List<Slint.DotNetValue>()
    });

    Slint.SetProperty(new Slint.DotNetValue
    {
        typeName = "img",
        typeType = 3,
        typeValue = "./ui/assets/toradex_logo.png",
        isStruct = false,
        structProps = new List<Slint.DotNetValue>()
    });

    return true;
});

Slint.NewTimer(1, 500, () =>
{
    Console.WriteLine("This was the timer...");
    return true;
});

new Thread(() =>
{
    while (true)
    {
        Console.WriteLine("Thread Id: {0}", Environment.CurrentManagedThreadId);
        Thread.Sleep(10000);

        Slint.RunOnUiThread(() =>
        {
            Console.WriteLine("Thread Id: {0}", Environment.CurrentManagedThreadId);
            Console.WriteLine("This was the UI thread...");

            Slint.SetProperty(new Slint.DotNetValue
            {
                typeName = "counter",
                typeType = 1,
                typeValue = 0.ToString(),
                isStruct = false,
                structProps = new List<Slint.DotNetValue>()
            });

            return true;
        });
    }
}).Start();

Console.WriteLine("Thread Id: {0}", Environment.CurrentManagedThreadId);
Slint.Run();
