
# SlintDotnet (Alpha)

[![npm](https://img.shields.io/nuget/v/SlintDotnet)](https://www.nuget.org/packages/SlintDotnet)

[Slint](https://slint.dev/) is a UI toolkit that supports different programming languages.
SlintDotnet is the integration with .NET C#.

> ⚠️ **This is experimental and not ready for production use!**
> SlintDotnet is still in the early stages of development: APIs will change and important features are still being developed.

## Installing Slint

Slint is available via Nuget package:

```bash
dotnet add package SlintDotnet
```

### Dependencies

You need to install the following components:

* Supported only on Linux (for now):
  * x64
  * arm
  * arm64
* [.NET 6.0 SDK for Linux](https://dotnet.microsoft.com/download/dotnet/6.0)
* fontconfig library (libfontconfig-dev on debian based distributions)

## Using SlintDotnet

There are a ready to use template from the [VS Code Torizon Templates](https://github.com/toradex/vscode-torizon-templates).

## API Overview

To have access to the Slint classes the following `using` statement is needed:

```cs
using Slint;
```

### Window Component

The window component from the `.slint` file is mapped to the `Window` class. To have access to the `Window` class is need to add the `using` statement to the namespace that is the same name of the `.slint` file. For example: if the `.slint` file is named `MyWindow.slint`:

```cs
using MyWindow;
```

Then the `Window` class can be instantiated and used:

```cs
var window = new Window();
window.run();
```

### Accessing a property

Properties are exposed as properties on the instance of the `Window`:

```cs
window.counter = 42;
```

> ⚠️ `struct` properties are accessed as properties on the instance of the `Window`

### Callbacks

The callbacks are also exposed as `Action` properties on the instance of the `Window`:

```cs
window.RequestIncreaseValue = () => {
    window.counter++;
};
```

> ⚠️ The keywords from the `.slint` file are converted to pascal case.

> ⚠️ Only `void(void)` callbacks are supported for now.

### Changing UI from Different Threads

The UI can only be changed from the UI thread, a panic will be triggered if you try to change the UI from a different thread. To change the UI from a different thread use:

```cs
  window.RunOnUiThread(() => {
    window.counter++;
  });
```

This will move the action to the Slint `upgrade_in_event_loop` to be executed in the UI thread.

> ⚠️ `Window.RunOnUiThread` can only be called after the `Window.Run` method. An exception will be thrown if called before.

### Type Mappings

| `.slint` Type     | C# Type       | Notes |
| ----------------- | ------------- | ----- |
| `int`             | `Float`       |       |
| `float`           | `Float`       |       |
| `string`          | `String`      |       |
| `bool`            | `bool`        |       |
| `image`           | `Slint.Image` |       |
| `Timer`           | `Slint.Timer` |       |
| `color`           | ❌            |       |
| `length`          | `Float`       |       |
| `physical_length` | ❌            |       |
| `duration`        | ❌            |       |
| `angle`           | ❌            |       |
| `struct`          | `object`      |       |
| `array`           | `List`            |    Only arrays of strings supported   |
