

/// Returned by the user callback given to the `EventsLoop::run` method.
///
/// Indicates whether the `run` method should continue or complete.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ControlFlow {
    /// Continue looping and waiting for events.
    Continue,
    /// Break from the event loop.
    Break,
}

/// Provides a way to retrieve events from the system and from the windows that were registered to
/// the events loop.
///
/// An `EventsLoop` can be seen more or less as a "context". Calling `EventsLoop::new()`
/// initializes everything that will be required to create windows. For example on Linux creating
/// an events loop opens a connection to the X or Wayland server.
///
/// To wake up an `EventsLoop` from a another thread, see the `EventsLoopProxy` docs.
///
/// Note that the `EventsLoop` cannot be shared accross threads (due to platform-dependant logic
/// forbiding it), as such it is neither `Send` nor `Sync`. If you need cross-thread access, the
/// `Window` created from this `EventsLoop` _can_ be sent to an other thread, and the
/// `EventsLoopProxy` allows you to wakeup an `EventsLoop` from an other thread.
pub trait EventsLoop<Event> {
    /// Fetches all the events that are pending, calls the callback function for each of them,
    /// and returns.
    fn poll_events<F>(&mut self, callback: F)
        where F: FnMut(Event);

    /// Calls `callback` every time an event is received. If no event is available, sleeps the
    /// current thread and waits for an event. If the callback returns `ControlFlow::Break` then
    /// `run` will immediately return.
    ///
    /// # Danger!
    ///
    /// The callback is run after *every* event, so if its execution time is non-trivial the event queue may not empty
    /// at a sufficient rate. Rendering in the callback with vsync enabled **will** cause significant lag.
    fn run<F>(&mut self, callback: F)
        where F: FnMut(Event) -> ControlFlow;

    /// Creates an `EventsLoopProxy` that can be used to wake up the `EventsLoop` from another
    /// thread.
    fn create_proxy(&self) -> Box<EventsLoopProxy>;
}
/// Used to wake up the `EventsLoop` from another thread.
pub trait EventsLoopProxy : Send {
    /// Wake up the `EventsLoop` from which this proxy was created.
    ///
    /// This causes the `EventsLoop` to emit an `Awakened` event.
    ///
    /// Returns an `Err` if the associated `EventsLoop` no longer exists.
    fn wakeup(&self) -> Result<(), EventsLoopClosed>;

    fn clone(&self) -> Box<EventsLoopProxy>;
}

impl Clone for Box<EventsLoopProxy> {
    fn clone(&self) -> Box<EventsLoopProxy> {
        use std::ops::Deref;
        self.deref().clone()
    }    
}

/// The error that is returned when an `EventsLoopProxy` attempts to wake up an `EventsLoop` that
/// no longer exists.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EventsLoopClosed;

impl std::fmt::Display for EventsLoopClosed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", std::error::Error::description(self))
    }
}

impl std::error::Error for EventsLoopClosed {
    fn description(&self) -> &str {
        "Tried to wake up a closed `EventsLoop`"
    }
}


#[cfg(test)]
mod tests {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum Events {
        A, B, C        
    }

    struct Loop;

    impl ::EventsLoop<Events> for Loop {
        fn poll_events<F>(&mut self, callback: F)
            where F: FnMut(Events) {
            let mut c = callback;
            c(Events::A);
            c(Events::B);
        }

        fn run<F>(&mut self, callback: F)
            where F: FnMut(Events) -> ::ControlFlow {
                let mut c = callback;
                while c(Events::C) == ::ControlFlow::Continue {

                }
            }

        fn create_proxy(&self) -> Box<::EventsLoopProxy> {
            unimplemented!();
        }

    }

    #[test]
    fn it_works() {
        use ::EventsLoop;
        let mut l = Loop{};
        l.poll_events(|e| println!("{:?}", e));
        l.run(|e| {println!("{:?}", e); ::ControlFlow::Break});
    }
}
