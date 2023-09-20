use std::{
    path::Path,
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
    Timer,
    TimerMode
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
    IMAGE = 3
}

#[derive(Net)]
pub struct DotNetValue {
    type_name: String,
    type_type: i32,
    type_value: String
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

#[net]
pub fn interprete(path: &str) -> Tokens {
    let mut compiler = slint_interpreter::ComponentCompiler::default();
    let path = std::path::Path::new(path);
    let ret_handle = async_std::task::block_on(compiler.build_from_path(path)).unwrap();

    let mut m_props: Vec<DotNetValue> = Vec::new();
    let props = ret_handle.properties();

    for prop in props {
        let p_name = prop.0;
        let p_type = prop.1;
        let val_type;
        let val_val = format!(
            "{:?}",
            ""
        );

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
            ValueType::Image => {
                val_type = DotNetType::IMAGE;
            },
            _ => {
                panic!("Slint type not supported");
            }
        }

        m_props.push(DotNetValue {
            type_name: p_name,
            type_type: val_type as i32,
            type_value: val_val
        });
    }

    let m_calls = ret_handle.callbacks().collect();

    let tokens = Tokens {
        props: m_props,
        calls: m_calls
    };

    tokens
}

#[net]
pub fn create(path: &str) {
    printdebug!("create()");

    let mut compiler = slint_interpreter::ComponentCompiler::default();
    let path = std::path::Path::new(path);
    let ret_handle = async_std::task::block_on(compiler.build_from_path(path)).unwrap();

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

#[net]
pub fn get_properties() -> Vec<DotNetValue> {
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
            let val_val = format!(
                "{:?}",
                strong_ref.get_property(&p_name).unwrap()
            );

            printdebug!("{}", val_val);

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
                _ => {
                    panic!("Slint type not supported");
                }
            }

            ret.push(DotNetValue {
                type_name: p_name,
                type_type: val_type as i32,
                type_value: val_val
            });
        }
    });

    ret
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
pub fn get_property(name: &str) -> DotNetValue {
    printdebug!("get_property()");

    let mut ret: DotNetValue = DotNetValue {
        type_name: "".to_string(),
        type_type: 0,
        type_value: "".to_string()
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
pub fn run() {
    printdebug!("run()");

    CURRENT_INSTANCE.with(|current| {
        let strong_ref = current.borrow_mut().take().unwrap();
        current.replace(Some(strong_ref.clone_strong()));

        strong_ref.run().unwrap();
    });
}
