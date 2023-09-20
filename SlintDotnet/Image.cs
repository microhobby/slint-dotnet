
namespace Slint;

public class Image {
    public static Image FromFile(string path) {
        if (!File.Exists(path)) {
            throw new FileNotFoundException();
        }

        var img = new Image();
        img.Path = path;
    
        return img;
    }

    public string? Path {
        get;
        set;
    }
}
