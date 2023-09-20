using System.Text;
using SlintAPI = SlintDotnet.SlintDotnet;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Text;
using System.Runtime.InteropServices;

namespace SlintDotnet.SourceGenerator;

[Generator]
public class Generator : ISourceGenerator
{
    public void Execute(GeneratorExecutionContext context)
    {
        // FUCK OFF ROSLYN
        var home = Environment.GetEnvironmentVariable("HOME");
        var arch = RuntimeInformation.OSArchitecture.ToString().ToLowerInvariant();

        string assemblyProbeDirectory = $"{home}/.nuget/packages/slintdotnet/1.2.19/runtimes/linux-{arch}/native/"; 
        Directory.SetCurrentDirectory(assemblyProbeDirectory);
    
        var sourceCodeStrWin = new StringBuilder("");
        // get the context file without the extension and path
        var path = context.AdditionalFiles
            .Single(t => t.Path.EndsWith(".slint"))
            .Path;
        // get the slint filename
        var fileName = Path.GetFileNameWithoutExtension(path);

        var tokens = SlintAPI.Interprete(path);

// add the namespace and class
sourceCodeStrWin.Append($@"
using Slint;
using SlintAPI = SlintDotnet.SlintDotnet;

namespace {fileName};

public class Window
{{
    private string _slintFile = ""./ui/{fileName}.slint"";
");

        var properties = tokens.props;

        foreach (var property in properties)
        {
            var valType = property.typeType switch
            {
                0 => "string",
                1 => "float",
                2 => "bool",
                3 => "Image",
                _ => "string"
            };

            var valTypeRust = property.typeType switch
            {
                0 => "String",
                1 => "Number",
                2 => "Bool",
                3 => "Image",
                _ => "String"
            };

// add the properties
sourceCodeStrWin.Append($@"

    private {valType} _{property.typeName};
    public {valType} {property.typeName} {{
        get {{
");

            if (valType != "Image") {
sourceCodeStrWin.Append($@"
            var rT = SlintAPI.GetProperty(""{property.typeName}"")
                        .typeValue
                        .Replace(""Value::{valTypeRust}("", """")
                        .Replace("")"", """");

");
            } else {
sourceCodeStrWin.Append($@"
            var rT = _{property.typeName};
");
            }

            if (valType == "string") {
sourceCodeStrWin.Append($@"
            return rT;
");
            } else if (valType == "float") {
sourceCodeStrWin.Append($@"
                return float.Parse(rT);
");
            } else if (valType == "bool") {
sourceCodeStrWin.Append($@"
                return bool.Parse(rT);
");
            } else {
sourceCodeStrWin.Append($@"
                return rT;
");
            }

sourceCodeStrWin.Append($@"
        }}

        set {{
            _{property.typeName} = value;
            SlintAPI.SetProperty(new SlintAPI.DotNetValue {{
                typeName = ""{property.typeName}"",
");

            if (valType == "string") {
sourceCodeStrWin.Append($@"
                typeType = 0,
                typeValue = value.ToString()
");
            } else if (valType == "float") {
sourceCodeStrWin.Append($@"
                typeType = 1,
                typeValue = value.ToString()
");
            } else if (valType == "bool") {
sourceCodeStrWin.Append($@"
                typeType = 2,
                typeValue = value.ToString()
");
            } else if (valType == "Image") {
sourceCodeStrWin.Append($@"
                typeType = 3,
                typeValue = value.Path
");
            } else {
sourceCodeStrWin.Append($@"
                typeType = 0,
                typeValue = value.ToString()
");
            }

sourceCodeStrWin.Append($@"
            }});
        }}
    }}

");
        }

        var methods = tokens.calls;

        foreach (var method in methods)
        {
            // convert to camel case
            var parts = method.Split('-');
            var camelName = "";
            foreach (var part in parts) {
                camelName += Char.ToUpperInvariant(part[0]) + part.Substring(1);
            }

// methods
sourceCodeStrWin.Append($@"
    private Action _{camelName};
    public Action {camelName} {{
        set {{
            _{camelName} = value;
            SlintAPI.SetCallback(""{method}"", () => {{
                _{camelName}?.Invoke();
                return true;
            }});
        }}
    }}
");

        }

// constructor and run
sourceCodeStrWin.Append($@"

    public Window()
    {{
        Console.WriteLine(""Hello from {fileName}!"");
        SlintAPI.Create(_slintFile);
    }}

    public void Run()
    {{
        SlintAPI.Run();
    }}
");


// end
sourceCodeStrWin.Append($@"
}}
");

        var sourceCode = SourceText.From(
            sourceCodeStrWin.ToString(),
            Encoding.UTF8
        );

        context.AddSource($"{fileName}.g.cs", sourceCode);
    }

    public void Initialize(GeneratorInitializationContext context)
    {
    }
}
