
namespace Slint;

public enum SlintType {
    STRING = 0,
    NUMBER = 1,
    BOOL = 2,
    IMAGE = 3,
    STRUCT = 4,
    ARRAY = 5
}


public interface SlintInitialization {
    void Init();
}

public class SlintInitializationPool {
    public static List<SlintInitialization> POOL = new();
}
