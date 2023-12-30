using System.Text;
using SlintAPI = SlintDotnet.SlintDotnet;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Text;
using System.Runtime.InteropServices;

namespace SlintDotnet.SourceGenerator;

[Generator]
public class Generator : ISourceGenerator
{
    // TODO: do not forget to update the version
    private static string PACKAGE_VERSION = "1.2.26";

    protected struct struct_info {
        public int index;
        public string struct_name;
    };

    private string CreateProperties (List<SlintAPI.DotNetValue> props, struct_info? strtI)
    {
        var sourceCodeStr = new StringBuilder("");
        foreach (var property in props)
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
sourceCodeStr.Append($@"

    private {valType} _{property.typeName};
    public {valType} {property.typeName} {{
        get {{
");

            if (strtI != null) {
                // for fields of the struct we need to refresh it from Slint
sourceCodeStr.Append($@"
            var sT = SlintAPI.GetStruct(""{strtI.Value.struct_name}"");
");
            } else if (valType != "Image") {
                // for the primitive types
sourceCodeStr.Append($@"
            var rT = SlintAPI.GetProperty(""{property.typeName}"")
                        .typeValue
                        .Replace(""Value::{valTypeRust}("", """")
                        .Replace("")"", """");

");
            } else {
                // for image only, return the Image object
sourceCodeStr.Append($@"
            var rT = _{property.typeName};
");
            }

            if (strtI != null) {
                // field of struct
                // prepare it
sourceCodeStr.Append($@"
             var rT = sT.structProps
                        .FirstOrDefault(p => p.typeName == ""{property.typeName}"")
                        .typeValue
                        .Replace(""Value::{valTypeRust}("", """")
                        .Replace("")"", """");
");
            }

            if (valType == "string") {
sourceCodeStr.Append($@"
            return rT;
");
            } else if (valType == "float") {
sourceCodeStr.Append($@"
                return float.Parse(rT);
");
            } else if (valType == "bool") {
sourceCodeStr.Append($@"
                return bool.Parse(rT);
");
            } else if (valType == "class") {
sourceCodeStr.Append($@"
            var sT = SlintAPI.GetStruct(""{property.typeName}"");
            this._{valType} = sT;
            return this._{valType};
");
            } else {
sourceCodeStr.Append($@"
                return rT;
");
            }

sourceCodeStr.Append($@"
        }}

");

sourceCodeStr.Append($@"
        set {{
            _{property.typeName} = value;
");

        if (strtI == null) {

sourceCodeStr.Append($@"
            SlintAPI.SetProperty(new SlintAPI.DotNetValue {{
                typeName = ""{property.typeName}"",
");
        } else {
sourceCodeStr.Append($@"
            SlintAPI.SetStruct(new SlintAPI.DotNetValue
            {{
                typeName = ""{strtI.Value.struct_name}"",
                typeType = 4,
                isStruct = true,
                typeValue = """",
                structProps = new List<SlintAPI.DotNetValue>
                {{
                    new SlintAPI.DotNetValue
                    {{
                        typeName = ""{property.typeName}"",
 ");
        }

        if (valType == "string") {
sourceCodeStr.Append($@"
                typeType = 0,
                typeValue = value.ToString(),
                isStruct = false,
                structProps = new List<SlintAPI.DotNetValue>()
");
        } else if (valType == "float") {
sourceCodeStr.Append($@"
                typeType = 1,
                typeValue = value.ToString(),
                isStruct = false,
                structProps = new List<SlintAPI.DotNetValue>()
");
        } else if (valType == "bool") {
sourceCodeStr.Append($@"
                typeType = 2,
                typeValue = value.ToString(),
                isStruct = false,
                structProps = new List<SlintAPI.DotNetValue>()
");
        } else if (valType == "Image") {
sourceCodeStr.Append($@"
                typeType = 3,
                typeValue = value.Path,
                isStruct = false,
                structProps = new List<SlintAPI.DotNetValue>()
");
        } else {
sourceCodeStr.Append($@"
                typeType = 0,
                typeValue = value.ToString(),
                isStruct = false,
                structProps = new List<SlintAPI.DotNetValue>()
");
        }

        if (strtI != null) {
sourceCodeStr.Append($@"
                }}
            }}
");
        }

sourceCodeStr.Append($@"
            }});
        }}
    }}

");
        }

        return sourceCodeStr.ToString();
    }

    private string CreateStructProperties (List<SlintAPI.DotNetValue> props)
    {
        var sourceCodeStr = new StringBuilder("");
        var struct_index = 0;

        foreach (var prop in props)
        {
sourceCodeStr.Append($@"

    private struct{struct_index} _{prop.typeName};
    public struct{struct_index} {prop.typeName} {{
        get {{
            var sT = SlintAPI.GetProperty(""{prop.typeName}"");

        }}
");

            struct_index++;
        }

        return sourceCodeStr.ToString();
    }

    public void Execute(GeneratorExecutionContext context)
    {
        // FUCK OFF ROSLYN
        var home = Environment.GetEnvironmentVariable("HOME");
        var arch = RuntimeInformation.OSArchitecture.ToString().ToLowerInvariant();

        string assemblyProbeDirectory = $"{home}/.nuget/packages/slintdotnet/{PACKAGE_VERSION}/runtimes/linux-{arch}/native/";
        Directory.SetCurrentDirectory(assemblyProbeDirectory);

        var sourceCodeStrWin = new StringBuilder("");
        // get the context file without the extension and path
        var path = context.AdditionalFiles
            .Single(t => t.Path.EndsWith(".slint"))
            .Path;
        // get the slint filename
        var fileName = Path.GetFileNameWithoutExtension(path);

        var tokens = SlintAPI.Interprete(path);
        var structs = tokens.props.Where(
            p => p.isStruct == true
        ).ToList();
        var props = tokens.props.Where(
            p => p.isStruct == false
        ).ToList();

// add the namespace and class
sourceCodeStrWin.Append($@"
using System.Linq;
using Slint;
using SlintAPI = SlintDotnet.SlintDotnet;

namespace {fileName};

");

sourceCodeStrWin.Append($@"
public class Window
{{
    private static bool _MAIN_RUNNING = false;
    private string _slintFile = ""./ui/{fileName}.slint"";

    public void RunOnUiThread (Action action)
    {{
        if (!Window._MAIN_RUNNING) {{
            throw new Exception(""You can only call Window.RunOnUiThread after call Window.Run"");
        }}

        SlintAPI.RunOnUiThread(() => {{
            action();
            return true;
        }});
    }}

");

            var struct_index = 0;
        foreach (var struc in structs)
        {
sourceCodeStrWin.Append($@"

    public class struct{struct_index}
    {{

");

            var strI = new struct_info {
                index = struct_index,
                struct_name = $"{struc.typeName}"
            };
            sourceCodeStrWin.Append(CreateProperties(struc.structProps, strI));

sourceCodeStrWin.Append($@"
    }}


    private struct{struct_index} _{struc.typeName} = new struct{struct_index}();
    public struct{struct_index} {struc.typeName}
    {{
        get {{
            return _{struc.typeName};
        }}
    }}
");

            struct_index++;
        }

        sourceCodeStrWin.Append(CreateProperties(props, null));

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
        Window._MAIN_RUNNING = true;
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
