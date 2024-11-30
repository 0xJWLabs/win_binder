[![Crate](https://img.shields.io/crates/v/win_binder.svg)](https://crates.io/crates/win_binder)

# win_binder

A simple library to listen and send events **globally** for keyboard and mouse on Windows.

Inspired by [rdev](https://github.com/Narsil/rdev), but built with [windows-rs](https://github.com/microsoft/windows-rs) instead of [winapi-rs](https://github.com/retep998/winapi-rs).

## Listening to Global Events

```rust
use win_binder::{listen, Event};

// This will block.
if let Err(error) = listen(callback) {
    println!("Error: {:?}", error)
}

fn callback(event: Event) {
    println!("My callback {:?}", event);
    match event.name {
        Some(string) => println!("User wrote {:?}", string),
        None => (),
    }
}
```

## Sending Events

```rust
use win_binder::{simulate, Button, EventType, Key, SimulateError};
use std::{thread, time};

fn send(event_type: &EventType) {
    let delay = time::Duration::from_millis(20);
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }
    // Let the OS catch up;
    thread::sleep(delay);
}

send(&EventType::KeyPress(Key::KeyS));
send(&EventType::KeyRelease(Key::KeyS));

send(&EventType::MouseMove { x: 0.0, y: 0.0 });
send(&EventType::MouseMove { x: 400.0, y: 400.0 });
send(&EventType::ButtonPress(Button::Left));
send(&EventType::ButtonRelease(Button::Right));
send(&EventType::Wheel {
    delta_x: 0,
    delta_y: 1,
});
```
## Main Structs
### Event

`Event` represents the event data that is received. It contains the `name` of the key or action as interpreted by the OS.
In order to detect what a user types, we need to plug to the OS level management
of keyboard state (modifiers like shift, CTRL, but also dead keys if they exist).

`EventType` corresponds to a *physical* event, corresponding to QWERTY layout
`Event` represents the event data that is received. It contains the `name` of the key
or action as interpreted by the OS and it will respect the layout.

```rust
/// When events arrive from the system we can add some information
/// time is when the event was received.
#[derive(Debug)]
pub struct Event {
    pub time: SystemTime,
    pub name: Option<String>,
    pub event_type: EventType,
}
```

Be careful, Event::name, might be None, but also String::from(""), and might contain
not displayable Unicode characters. We send exactly what the OS sends us, so do some sanity checking
before using it.

### EventType

In order to manage different OS, the current EventType choices is a mix and match to account for all possible events.
There is a safe mechanism to detect events no matter what, which are the
Unknown() variant of the enum which will contain some OS specific value.
Also, not that not all keys are mapped to an OS code, so simulate might fail if you
try to send an unmapped key. Sending Unknown() variants will always work (the OS might
still reject it).

```rust
/// In order to manage different OS, the current EventType choices is a mix&match
/// to account for all possible events.
#[derive(Debug)]
pub enum EventType {
    /// The keys correspond to a standard qwerty layout, they don't correspond
    /// To the actual letter a user would use, that requires some layout logic to be added.
    KeyPress(Key),
    KeyRelease(Key),
    /// Some mouse will have more than 3 buttons, these are not defined, and different OS will
    /// give different Unknown code.
    ButtonPress(Button),
    ButtonRelease(Button),
    /// Values in pixels
    MouseMove {
        x: f64,
        y: f64,
    },
    Wheel {
        delta_x: i64,
        delta_y: i64,
    },
}
```


## Getting the Main Screen Size

```rust
use win_binder::{display_size};

let (w, h) = display_size().unwrap();
assert!(w > 0);
assert!(h > 0);
```

## Keyboard state

We can define a dummy Keyboard, that we will use to detect
what kind of `EventType` trigger some String. We get the currently used
layout for now !
Caveat : This is layout dependent. If your app needs to support
layout switching, don't use this!
Caveat: Only shift and dead keys are implemented, Alt+Unicode code on Windows won't work.

```rust
use win_binder::{Keyboard, EventType, Key, KeyboardState};

let mut keyboard = Keyboard::new().unwrap();
let string = keyboard.add(&EventType::KeyPress(Key::KeyS));
// string == Some("s")
```

## Grabbing Global Events. (Requires `unstable_grab` Feature)

Installing this library with the `unstable_grab` feature adds the `grab` function
which hooks into the global input device event stream.
By supplying this function with a callback, you can intercept
all keyboard and mouse events before they are delivered to applications / window managers.
In the callback, returning None ignores the event and returning the event lets it pass.
There is no modification of the event possible here (yet).

Note: the use of the word `unstable` here refers specifically to the fact that the `grab` API is unstable and subject to change

```rust
#[cfg(feature = "unstable_grab")]
use win_binder::{grab, Event, EventType, Key};

#[cfg(feature = "unstable_grab")]
let callback = |event: Event| -> Option<Event> {
    if let EventType::KeyPress(Key::CapsLock) = event.event_type {
        println!("Consuming and cancelling CapsLock");
        None  // CapsLock is now effectively disabled
    }
    else { Some(event) }
};
// This will block.
#[cfg(feature = "unstable_grab")]
if let Err(error) = grab(callback) {
    println!("Error: {:?}", error)
}
```

## Serialization

Event data returned by the `listen` and `grab` functions can be serialized and deserialized with
Serde if you install this library with the `serialize` feature.
