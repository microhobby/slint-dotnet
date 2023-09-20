using SlintAPI = SlintDotnet.SlintDotnet;

namespace Slint;

public enum TimerMode {
    SingleShot,
    Repeated
}

public class Timer {
    public SlintAPI.DotNetTimer RustTimer {
        set;
        get;
    }

    public static Timer Start(
        TimerMode mode, ulong interval, Action callback
    ) {
        var tRet = new Timer();
        var tId = SlintAPI.NewTimer((int)mode, interval, () => {
            callback.Invoke();
            return true;
        });

        tRet.RustTimer = tId;

        return tRet;
    }

    public Timer() {
    }
}
