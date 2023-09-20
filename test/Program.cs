﻿using Slint = SlintDotnet.SlintDotnet;

Console.WriteLine("Hello, World!");

Console.WriteLine("Creating...");
Slint.Create("/home/castello/tmp/slint-dotnet/test/ui/appwindow.slint");
Console.WriteLine("Created");

Console.WriteLine("Getting properties...");
var props = Slint.GetProperties();
Console.WriteLine("Geted");

foreach(var prop in props) {
    Console.WriteLine($"Name: {prop.typeName}");
    Console.WriteLine($"Type: {prop.typeType}");
    Console.WriteLine($"Val: {prop.typeValue}");
}

Slint.SetProperty(new Slint.DotNetValue {
    typeName = "img",
    typeType = 3,
    typeValue = "./ui/assets/torizon_logo_white.svg"
});

Slint.SetCallback("request-increase-value", () => {
    var dt = Slint.GetProperty("counter");
    var sT = dt.typeValue
                .Replace("Value::Number(", "")
                .Replace(")", "");
    var val = float.Parse(sT) + 1;
    
    Slint.SetProperty(new Slint.DotNetValue {
        typeName = "counter",
        typeType = 1,
        typeValue = val.ToString()
    });

    Slint.SetProperty(new Slint.DotNetValue {
        typeName = "img",
        typeType = 3,
        typeValue = "./ui/assets/toradex_logo.png"
    });
    
    return true;
});

Slint.NewTimer(1, 500, () => {
    Console.WriteLine("This was the timer...");
    return true;
});

Slint.Run();
