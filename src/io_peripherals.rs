//! Traits for [`Input`] and [`Output`] Peripherals that let controllers (like
//! the tui) write and read from them (respectively).
//!
//! [`Input`]: `lc3_traits::peripherals::Input`
//! [`Output`]: `lc3_traits::peripherals::Output`

use lc3_shims::peripherals::{Sink, SourceShim};

use std::io::{Read, Write};
use std::sync::Mutex;

/// A trait for [`Input`] Peripherals that lets us, a controller, supply the
/// inputs to the peripheral.
///
/// This is useful for virtual input peripherals like the [`InputShim`] and for
/// other situations where the input peripheral is designed to behave like a
/// serial port or a tty that the controller can use to communicate with the
/// device (i.e. the UART backed `Input`/`Output` peripherals used by default on
/// boards).
///
/// Note that this is already implemented for the default input source
/// ([`SourceShim`]) used by the input shim ([`InputShim`]) but not for all
/// input sources that can be used with [`InputShim`]. If you create your own
/// input source that you implement [`Source`] for, be sure to implement this
/// trait for it as well if you wish to use your input source with a controller
/// application (like the tui).
///
/// Note that this is not a super trait for [`Source`] because it is possible to
/// have [`Input`] implementations that are not the [`InputShim`] communicate
/// with the controller.
///
/// [`Input`]: `lc3_traits::peripherals::Input`
/// [`InputShim`]: `lc3_shims::peripherals::InputShim`
/// [`SourceShim`]: `lc3_shims::peripherals::SourceShim`
/// [`Source`]: `lc3_shims::peripherals::Source`
pub trait InputSink {
    // Note: probably only ASCII for now.
    //
    // Should return `None` on errors/invalid chars.
    fn put_char(&self, c: char) -> Option<()>;
}

/// A trait for [`Output`] Peripherals that lets us, a controller, consume the
/// outputs from the peripheral.
///
/// This is the [`InputSink`] trait's [`Output`] peripheral counterpart.
///
/// Analogous to the [`InputSink`] trait, this is useful for virtual output
/// peripherals like the [`OutputShim`] and for other situations where the
/// output peripheral is designed to behave like a serial port or a tty that the
/// controller can use to communicate with the device (i.e. the UART backed
/// `Input`/`Output` peripherals used by default on boards).
///
/// Note that this is already implemented for the default output source
/// (a `Mutex<Vec<u8>>`) used by the output shim ([`OutputShim`]) as well as
/// every `Mutex` based [`Sink`] whose inner type implements both [`Read`] and
/// [`Write`].
///
/// In the unlikely event that you find yourself creating your own output sink
/// that you implement [`Sink`] for (or if your `Mutex` based sink's inner type
/// supports [`Write`] but not [`Read`]), be sure to implement this trait for it
/// as well if you wish to use your input source with a controller application
/// (like the tui).
///
/// Note that this is not a super trait for [`Sink`] because it is possible to
/// have [`Output`] implementations that are not the [`OutputShim`] communicate
/// with the controller (i.e. UART backed peripherals as mentioned above).
///
/// [`Output`]: `lc3_traits::peripherals::Output`
/// [`OutputShim`]: `lc3_shims::peripherals::OutputShim`
/// [`Sink`]: `lc3_shims::peripherals::Sink`
/// [`Read`]: `std::io::Read`
/// [`Write`]: `std::io::Write`
pub trait OutputSource {
    // Note: probably only ASCII for now.
    //
    // Should return `None` when no characters are available.
    fn get_chars(&self) -> Option<String>;
}


// This is fine!
impl InputSink for &SourceShim {
    fn put_char(&self, c: char) -> Option<()> {
        self.push(c);
        Some(())
    }
}

// This is less fine.. (should maybe be as generic as the Sink trait, but that
// is not trivial) (TODO)
impl OutputSource for Mutex<Vec<u8>> {
    fn get_chars(&self) -> Option<String> {
        // This is bad, maybe:
        let mut vec = self.lock().unwrap();
        if !vec.is_empty() {
            let v = std::mem::replace(vec.deref_mut(), Vec::new());

            // TODO: maybe handle non-utf8 char better than this.
            String::from_utf8(v).ok()
        } else {
            None
        }
    }
}

// Mirrors the blanket impl that `Sink` has but also requires `Read` support so
// that we can actually implement OutputSource.
impl<W: Read + Write> OutputSource for Mutex<W>
where
    Mutex<W>: Sink // This is really guaranteed.
{
    fn get_chars(&self) -> Option<String> {
        let mut buf = Vec::new();
        let mut source = self.lock().unwrap();

        if let Some(n) = source.read_to_end(&mut buf) {
            if n > 0 {
                // TODO: maybe handle non-utf8 chars better than this.
                String::from_utf8(buf).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}
