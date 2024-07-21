use std::{
    path::Path,
    rc::Rc,
    sync::Once,
    time::Duration
};
use rnet::{
    Net,
    net,
    Delegate0
};
use slint::{
    ComponentHandle,
    Image,
    Model,
    ModelRc, Timer, TimerMode, VecModel, Weak
};
use slint_interpreter::{
    ComponentInstance,
    ValueType,
    Value
};

rnet::root!();

macro_rules! printdebug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}

enum DotNetType {
    STRING = 0,
    NUMBER = 1,
    BOOL = 2,
    IMAGE = 3,
    STRUCT = 4,
    ARRAY = 5
}

#[derive(Net)]
pub struct DotNetValue {
    type_name: String,
    type_type: i32,
    type_value: String,
    is_struct: bool,
    is_array: bool,
    struct_props: Vec<DotNetValue>,
    array_items: Vec<DotNetValue>
}

#[derive(Net)]
pub struct Tokens {
    props: Vec<DotNetValue>,
    calls: Vec<String>
}

#[derive(Net)]
pub struct DotNetTimer {
    timer_id: i32,
    interval: u64
}

thread_local! {
    static TIMER_POOL: std::cell::RefCell<Option<Vec<Timer>>> = Default::default();
}

thread_local! {
    static CURRENT_INSTANCE: std::cell::RefCell<Option<ComponentInstance>> = Default::default();
}

// reject modernity, back to the monke
static mut MAIN_WEAK_INSTANCE: Option<Weak<ComponentInstance>> = None;

#[net]
pub fn create(path: &str) {
    printdebug!("create()");

    let mut compiler = slint_interpreter::ComponentCompiler::default();
    let path = std::path::Path::new(path);
    let compiler_ret = async_std::task::block_on(compiler.build_from_path(path));

    // check if we had some Slint lang error
    if compiler.diagnostics().len() > 0 {
        slint_interpreter::print_diagnostics(compiler.diagnostics());
        panic!("Slint compilation failed");
    }

    let ret_handle = compiler_ret.unwrap();

    slint_interpreter::print_diagnostics(compiler.diagnostics());
    let component = ret_handle.create().unwrap();

    CURRENT_INSTANCE.with(
        |current|
            current.replace(Some(component.clone_strong()))
    );

    TIMER_POOL.with(| pool | {
        pool.replace(Some(Vec::new()));
    });
}

fn internal_get_properties_weak(component_weak: Weak<ComponentInstance>) -> Vec<DotNetValue> {
    CURRENT_INSTANCE.with(
        |current|
            current.replace(Some(component_weak.unwrap()))
    );

    internal_get_properties()
}

fn internal_get_properties() -> Vec<DotNetValue> {
    printdebug!("get_properties()");

    let mut ret: Vec<DotNetValue> = Vec::new();

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        let binding = strong_ref.definition();
        let props = binding.properties();

        for prop in props {
            let p_name = prop.0;
            let p_type = prop.1;
            let val_type;
            let mut val_struct = false;
            let mut val_array: bool = false;
            let mut val_props = Vec::new();
            let mut val_items = Vec::new();
            let val_val = format!(
                "{:?}",
                strong_ref.get_property(&p_name).unwrap()
            );

            printdebug!("property {} value {}", p_name, val_val);

            match p_type {
                ValueType::String => {
                    val_type = DotNetType::STRING;
                },
                ValueType::Number => {
                    val_type = DotNetType::NUMBER;
                },
                ValueType::Bool => {
                    val_type = DotNetType::BOOL;
                },
                ValueType::Image  => {
                    val_type = DotNetType::IMAGE;
                },
                ValueType::Model => {
                    val_type = DotNetType::ARRAY;
                    val_array = true;

                    let s_val = strong_ref.get_property(&p_name).unwrap();
                    match s_val {
                        Value::Model(arr) => {
                            for item in arr.iter() {
                                let s_name = "".to_string();
                                let s_type = item.value_type();
                                let sval_type;
                                let sval_struct = false;
                                let sval_array = false;
                                let sval_val = format!(
                                    "{:?}",
                                    item
                                );

                                printdebug!("array item value {}", sval_val);

                                match s_type {
                                    ValueType::String => sval_type = DotNetType::STRING,
                                    ValueType::Number => sval_type = DotNetType::NUMBER,
                                    ValueType::Bool => sval_type = DotNetType::BOOL,
                                    ValueType::Image => sval_type = DotNetType::IMAGE,
                                    ValueType::Struct => {
                                        panic!("struct inside array not supported");
                                    },
                                    _ => {
                                        panic!("Slint type not supported inside an array");
                                    }
                                }

                                val_items.push(DotNetValue {
                                    type_name: s_name,
                                    type_type: sval_type as i32,
                                    type_value: sval_val,
                                    is_struct: sval_struct,
                                    is_array: sval_array,
                                    struct_props: Vec::new(),
                                    array_items: Vec::new()
                                });
                            }
                        }
                        _ => {
                            panic!("unde'f'ined array type found ????");
                        }
                    }
                },
                ValueType::Struct => {
                    val_type = DotNetType::STRUCT;
                    val_struct = true;

                    // create the struct props
                    let s_val = strong_ref.get_property(&p_name).unwrap();
                    match s_val {
                        Value::Struct(stru) => {
                            for field in stru.iter() {
                                let s_name = field.0.to_string();
                                let s_type = field.1.value_type();
                                let sval_type;
                                let sval_struct = false;
                                let sval_val = format!(
                                    "{:?}",
                                    field.1
                                );

                                printdebug!("struct field {} value {}", s_name, sval_val);

                                // FIX-ME: for now we do not accept
                                // struct inside struct
                                match s_type {
                                    ValueType::String => sval_type = DotNetType::STRING,
                                    ValueType::Number => sval_type = DotNetType::NUMBER,
                                    ValueType::Bool => sval_type = DotNetType::BOOL,
                                    ValueType::Image => sval_type = DotNetType::IMAGE,
                                    ValueType::Struct => {
                                        panic!("struct inside struct not supported");
                                    },
                                    _ => {
                                        panic!("Slint type not supported inside a struct");
                                    }
                                }

                                val_props.push(DotNetValue {
                                    type_name: s_name,
                                    type_type: sval_type as i32,
                                    type_value: sval_val,
                                    is_struct: sval_struct,
                                    is_array: false,
                                    struct_props: Vec::new(),
                                    array_items: Vec::new()
                                });
                            }
                        }
                        _ => {
                            panic!("undefined struct type found ????");
                        }
                    }
                },
                _ => {
                    panic!("Slint type not supported");
                }
            }

            ret.push(DotNetValue {
                type_name: p_name,
                type_type: val_type as i32,
                type_value: val_val,
                is_struct: val_struct,
                is_array: val_array,
                struct_props: val_props,
                array_items: val_items
            });
        }
    });

    ret
}

#[net]
pub fn get_properties() -> Vec<DotNetValue> {
    internal_get_properties()
}

#[net]
pub fn set_struct(value: DotNetValue) {
    printdebug!("set_struct()");

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        let name = &value.type_name;
        let props = value.struct_props;
        let val = strong_ref.get_property(name).unwrap();

        match val {
            Value::Struct(mut stru) => {
                for field in stru.clone().iter() {
                    for from_dot_net in &props {
                        if field.0 == from_dot_net.type_name {
                            printdebug!("Field {} found, updating...", field.0);

                            if (DotNetType::STRING as i32) == from_dot_net.type_type {
                                stru.set_field(
                                    from_dot_net.type_name.clone().into(),
                                    Value::String(from_dot_net.type_value.clone().into())
                                );
                            }
                            else if (DotNetType::NUMBER as i32) == from_dot_net.type_type {
                                stru.set_field(
                                    from_dot_net.type_name.clone().into(),
                                    Value::Number(from_dot_net.type_value.parse::<f64>().unwrap())
                                );
                            }
                            else if (DotNetType::BOOL as i32) == from_dot_net.type_type {
                                let val = if from_dot_net.type_value == "True" {
                                    true
                                } else {
                                    false
                                };

                                stru.set_field(
                                    from_dot_net.type_name.clone().into(),
                                    Value::Bool(val)
                                );
                            }
                            else if (DotNetType::IMAGE as i32) == value.type_type {
                                let path = Path::new(&value.type_value);
                                let img = Image::load_from_path(path).unwrap();


                                stru.set_field(
                                    from_dot_net.type_name.clone().into(),
                                    Value::Image(img)
                                );
                            } else {
                                panic!("Type {} was not resolved", value.type_type);
                            }
                        }
                    }
                }

                // then set the struct back to the component
                strong_ref.set_property(name, Value::Struct(stru)).unwrap();
            }
            _ => {
                panic!("undefined struct type found or you are trying to access a non struct typep property");
            }
        }
    });
}

#[net]
pub fn set_property(value: DotNetValue) {
    printdebug!("set_property()");

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        if (DotNetType::STRING as i32) == value.type_type {
            strong_ref.set_property(
                &value.type_name,
                Value::String(value.type_value.into())
            ).unwrap();
        }
        else if (DotNetType::NUMBER as i32) == value.type_type {
            strong_ref.set_property(
                &value.type_name,
                Value::Number(value.type_value.parse::<f64>().unwrap())
            ).unwrap();
        }
        else if (DotNetType::BOOL as i32) == value.type_type {
            let val = if value.type_value == "True" {
                true
            } else {
                false
            };

            strong_ref.set_property(
                &value.type_name,
                Value::Bool(val)
            ).unwrap();
        }
        else if (DotNetType::IMAGE as i32) == value.type_type {
            let path = Path::new(&value.type_value);
            let img = Image::load_from_path(path).unwrap();

            strong_ref.set_property(
                &value.type_name,
                Value::Image(img)
            ).unwrap();
        } else {
            panic!("Type {} was not resolved", value.type_type);
        }
    });
}

#[net]
pub fn get_array(name: &str) -> DotNetValue {
    printdebug!("get_array()");

    let mut ret: DotNetValue = DotNetValue {
        type_name: "".to_string(),
        type_type: DotNetType::ARRAY as i32,
        type_value: "".to_string(),
        is_struct: false,
        is_array: true,
        struct_props: Vec::new(),
        array_items: Vec::new()
    };

    let mut val_items = Vec::new();

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        let val = strong_ref.get_property(name).unwrap();

        ret.type_name = name.into();
        // there is no "value"
        ret.type_value = "".to_string();

        // make sure that this is an array
        if val.value_type() == ValueType::Model {
            // get the array type
            match val {
                Value::Model(arr) => {
                    for item in arr.iter() {
                        let s_name = "".to_string();
                        let s_type = item.value_type();
                        let sval_type;
                        let sval_struct = false;
                        let sval_array = false;
                        let sval_val = format!(
                            "{:?}",
                            item
                        );

                        printdebug!("array item value {}", sval_val);

                        match s_type {
                            ValueType::String => sval_type = DotNetType::STRING,
                            ValueType::Number => sval_type = DotNetType::NUMBER,
                            ValueType::Bool => sval_type = DotNetType::BOOL,
                            ValueType::Image => sval_type = DotNetType::IMAGE,
                            ValueType::Struct => {
                                panic!("struct inside array not supported");
                            },
                            _ => {
                                panic!("Slint type not supported inside an array");
                            }
                        }

                        val_items.push(DotNetValue {
                            type_name: s_name,
                            type_type: sval_type as i32,
                            type_value: sval_val,
                            is_struct: sval_struct,
                            is_array: sval_array,
                            struct_props: Vec::new(),
                            array_items: Vec::new()
                        });
                    }
                }
                _ => {
                    panic!("undefined array type found ????");
                }
            }
        } else {
            panic!("This property is not an array");
        }
    });

    ret.array_items = val_items;
    ret
}

#[net]
pub fn set_array(value: DotNetValue) {
    printdebug!("set_array()");

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        let name = &value.type_name;
        let items = value.array_items;
        let s_items = VecModel::default();
        let mut ix = 0;

        for item in items {
            if (DotNetType::STRING as i32) == item.type_type {
                let mut str_val = item.type_value.replace("Value::String(\"", "");
                str_val = str_val.replace("\")", "");
                s_items.insert(ix, slint_interpreter::Value::String(str_val.into()));
                ix += 1;
            } else {
                panic!("Type {} was not resolved for array items", item.type_type);
            }
        }

        strong_ref.set_property(name, Value::Model(
            ModelRc::from(Rc::new(s_items).clone())
        )).unwrap();
    });
}

#[net]
pub fn get_struct(name: &str) -> DotNetValue {
    printdebug!("get_struct()");

    let mut ret: DotNetValue = DotNetValue {
        type_name: "".to_string(),
        type_type: 0,
        type_value: "".to_string(),
        is_struct: true,
        is_array: false,
        struct_props: Vec::new(),
        array_items: Vec::new()
    };

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        let val = strong_ref.get_property(name).unwrap();

        ret.type_name = name.into();
        ret.type_type = val.value_type() as i32;
        // there is no "value"
        ret.type_value = "".to_string();

        match val {
            Value::Struct(stru) => {
                for field in stru.iter() {
                    let s_name = field.0.to_string();
                    let s_type = field.1.value_type();
                    let sval_type;
                    let sval_struct = false;
                    let sval_val = format!(
                        "{:?}",
                        field.1
                    );

                    printdebug!("struct field {} value {}", s_name, sval_val);

                    // FIX-ME: for now we do not accept
                    // struct inside struct
                    match s_type {
                        ValueType::String => sval_type = DotNetType::STRING,
                        ValueType::Number => sval_type = DotNetType::NUMBER,
                        ValueType::Bool => sval_type = DotNetType::BOOL,
                        ValueType::Image => sval_type = DotNetType::IMAGE,
                        ValueType::Struct => {
                            panic!("struct inside struct not supported");
                        },
                        _ => {
                            panic!("Slint type not supported inside a struct");
                        }
                    }

                    ret.struct_props.push(DotNetValue {
                        type_name: s_name,
                        type_type: sval_type as i32,
                        type_value: sval_val,
                        is_struct: sval_struct,
                        is_array: false,
                        struct_props: Vec::new(),
                        array_items: Vec::new()
                    });
                }
            }
            _ => {
                panic!("undefined struct type found ????");
            }
        }
    });

    ret
}

#[net]
pub fn get_property(name: &str) -> DotNetValue {
    let mut ret: DotNetValue = DotNetValue {
        type_name: "".to_string(),
        type_type: 0,
        type_value: "".to_string(),
        is_struct: false,
        is_array: false,
        struct_props: Vec::new(),
        array_items: Vec::new()
    };

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        let val = strong_ref.get_property(name).unwrap();

        let val_str = format!("{:?}", val);
        ret.type_name = name.into();
        ret.type_type = val.value_type() as i32;
        ret.type_value = val_str.clone();

        printdebug!("{}", val_str);
    });

    ret
}

#[net]
pub fn get_callbacks() -> Vec<String> {
    printdebug!("get_callbacks()");

    let mut ret: Vec<String> = Vec::new();

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        let binding = strong_ref.definition();
        let calls = binding.callbacks();
        ret = calls.collect();
    });

    ret
}

#[net]
pub fn set_callback(name: &str, callback: Delegate0<bool>) {
    printdebug!("set_callback()");

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        strong_ref.set_callback(name, move |_| {
            callback.call();
            Value::Void
        }).unwrap();
    });
}

#[net]
pub fn call_callback(name: &str) {
    printdebug!("call_callback()");

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        strong_ref.invoke(name, &[]).unwrap();
    });
}

#[net]
pub fn new_timer(mode: i32, interval: u64, callback: Delegate0<bool>) -> DotNetTimer {
    let ret = DotNetTimer {
        timer_id: -1,
        interval
    };

    TIMER_POOL.with(| pool | {
        let mut mut_pool = pool.borrow_mut().take().unwrap();
        let timer = Timer::default();

        let time_mode = if mode == 1 {
            TimerMode::Repeated
        } else {
            TimerMode::SingleShot
        };

        let int_duration =
            Duration::from_millis(interval);

        timer.start(time_mode, int_duration, move | | {
            callback.call();
        });

        mut_pool.push(timer);
        pool.replace(Some(mut_pool));
    });

    ret
}

#[net]
pub fn stop_timer(timer: DotNetTimer) {
    TIMER_POOL.with(| pool | {
        let pool = pool.borrow_mut().take().unwrap();
        let mut index = 0;

        // for each timer check if the id match
        for tmr in pool {
            if index == timer.timer_id {
                tmr.stop();
                break;
            }

            index += 1;
        }
    });
}

#[net]
pub fn restart_timer(timer: DotNetTimer) {
    TIMER_POOL.with(| pool | {
        let pool = pool.borrow_mut().take().unwrap();
        let mut index = 0;

        // for each timer check if the id match
        for tmr in pool {
            if index == timer.timer_id {
                tmr.restart();
                break;
            }

            index += 1;
        }
    });
}

#[net]
pub fn run_on_ui_thread(callback: Delegate0<bool>) {
    printdebug!("run_on_ui_thread()");

    // reject modernity, back to the monke
    let weak_ref = unsafe {
        MAIN_WEAK_INSTANCE.take().unwrap()
    };
    unsafe {
        MAIN_WEAK_INSTANCE = Some(weak_ref.clone());
    };

    weak_ref.upgrade_in_event_loop(move |_| {
        callback.call();
    }).unwrap();
}

#[net]
pub fn run() {
    printdebug!("run()");

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));
        let weak_ref = strong_ref.as_weak();

        unsafe {
            MAIN_WEAK_INSTANCE = Some(weak_ref);
        };

        strong_ref.run().unwrap();
    });
}

// Fix the Roslyn source generator race condition
static INIT: Once = Once::new();
static mut COMPONENT: Option<ComponentInstance> = None;

#[net]
pub fn interprete(path: &str) -> Tokens {
    let mut compiler = slint_interpreter::ComponentCompiler::default();
    let path = std::path::Path::new(path);
    let compiler_ret = async_std::task::block_on(compiler.build_from_path(path));

    // check if we had some Slint lang error
    if compiler.diagnostics().len() > 0 {
        slint_interpreter::print_diagnostics(compiler.diagnostics());

        // we do not panic here because we are only developing
        // the roslyn source generator should not crash omnisharp server
        println!("Slint compilation failed");

        // return nothing
        return Tokens {
            props: Vec::new(),
            calls: Vec::new()
        };
    }

    // ok, we are good to go
    let ret_handle = compiler_ret.unwrap();

    INIT.call_once(|| {
        unsafe {
            COMPONENT = Some(ret_handle.create().unwrap());
        }
    });

    unsafe {
        let m_calls = COMPONENT.as_ref().unwrap().definition().callbacks().collect();
        let m_props = internal_get_properties_weak(COMPONENT.as_ref().unwrap().as_weak());

        let tokens = Tokens {
            props: m_props,
            calls: m_calls
        };

        tokens
    }
}
