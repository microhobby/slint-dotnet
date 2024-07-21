using Slint = SlintDotnet.SlintDotnet;

Console.WriteLine("Hello, World!");

Console.WriteLine("Creating...");
Slint.Create("./ui/appwindow.slint");
Console.WriteLine("Created");

Console.WriteLine("Getting properties...");
var dprop = Slint.GetProperty("testStruct");
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
    isArray = false,
    structProps = new List<Slint.DotNetValue>(),
    arrayItems = new List<Slint.DotNetValue>()
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
        isArray = false,
        typeValue = "",
        arrayItems = new List<Slint.DotNetValue>(),
        structProps = new List<Slint.DotNetValue>
        {
            new Slint.DotNetValue
            {
                typeName = "T1",
                typeType = 0,
                typeValue = "modified",
                isStruct = false,
                isArray = false,
                structProps = new List<Slint.DotNetValue>(),
                arrayItems = new List<Slint.DotNetValue>()
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
        isArray = false,
        structProps = new List<Slint.DotNetValue>(),
        arrayItems = new List<Slint.DotNetValue>()
    });

    Slint.SetProperty(new Slint.DotNetValue
    {
        typeName = "img",
        typeType = 3,
        typeValue = "./ui/assets/toradex_logo.png",
        isStruct = false,
        isArray = false,
        structProps = new List<Slint.DotNetValue>(),
        arrayItems = new List<Slint.DotNetValue>()
    });

    var list = Slint.GetArray("lista");
    list.arrayItems.Add(new Slint.DotNetValue
    {
        typeName = "",
        typeType = 0,
        typeValue = "dd",
        isStruct = false,
        isArray = false,
        structProps = new List<Slint.DotNetValue>(),
        arrayItems = new List<Slint.DotNetValue>()
    });
    // Slint.SetProperty(list);
    Slint.SetArray(list);

    var list2 = Slint.GetArray("lista");

    for (var i = 0; i < list2.arrayItems.Count; i++)
    {
        if (
            list.arrayItems[i].typeValue.Replace("Value::String(\"", "").Replace("\")", "") !=
            list2.arrayItems[i].typeValue.Replace("Value::String(\"", "").Replace("\")", "")
        ) throw new Exception("Error arrays not equal");
        Console.WriteLine($"Item {i}: {list.arrayItems[i].typeValue} == {list2.arrayItems[i].typeValue}");
    }

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
                isArray = false,
                structProps = new List<Slint.DotNetValue>(),
                arrayItems = new List<Slint.DotNetValue>()
            });

            return true;
        });
    }
}).Start();

Console.WriteLine("Thread Id: {0}", Environment.CurrentManagedThreadId);
Slint.Run();
