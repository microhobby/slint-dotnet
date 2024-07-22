using System.Threading;
using static SlintDotnet.SlintDotnet;
using SlintAPI = SlintDotnet.SlintDotnet;

namespace Slint;

public class Array<T> : List<T>, SlintInitialization {
    private DotNetValue _dotnetArray;
    private string _varName;
    private bool _isInitialized = false;

    public Array(string varName) : base() {
        _varName = varName;
        SlintInitializationPool.POOL.Add(this);
    }

    /**
     * This method should be called only after the Slint UI is ready
     */
    public void Init() {
        _dotnetArray = SlintAPI.GetArray(_varName);

        // check if the slint one already have items and update this one
        foreach (var item in _dotnetArray.arrayItems) {
            if (item.typeType == (int)SlintType.STRING) {
                // call the base because we do not need to call the Slint side
                base.Add((T)(object)item.typeValue);
            }
        }

        _isInitialized = true;
    }

    private void _syncData() {
        if (!_isInitialized) {
            throw new System.Exception("Array not initialized yet");
        }

        _dotnetArray.arrayItems.Clear();

        foreach (var item in this) {
            _dotnetArray.arrayItems.Add(new DotNetValue {
                arrayItems = new List<DotNetValue>(),
                isArray = false,
                isStruct = false,
                structProps = new List<DotNetValue>(),
                typeName = "",
                typeType = (int)SlintType.STRING,
                typeValue = item!.ToString()
            });
        }

        // update the slint side
        SlintAPI.SetArray(_dotnetArray);
    }

    public new void Add(T item) {
        base.Add(item);
        _syncData();
    }

    public new void Remove(T item) {
        base.Remove(item);
        _syncData();
    }
}
