use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};

// TODO: one method
pub trait Input: Send {
    fn read(&mut self) -> io::Result<Event>;

    fn read_timeout(&mut self, timeout: Duration) -> io::Result<Option<Event>>;
}

pub struct CrosstermInput;

impl Input for CrosstermInput {
    fn read(&mut self) -> io::Result<Event> {
        event::read()
    }

    fn read_timeout(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        if event::poll(timeout)? {
            return Ok(Some(event::read()?));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use std::{
        ops::BitOr,
        sync::mpsc::{Receiver, RecvTimeoutError, Sender},
    };

    use super::*;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use itertools::Itertools;
    use winnow::{
        ascii::{alpha1, escaped_transform},
        combinator::{alt, delimited, dispatch, empty, fail, not, preceded, repeat, terminated},
        token::{any, none_of, take},
        ModalResult, Parser,
    };

    use super::Input;

    impl Input for Receiver<TestEvent> {
        fn read(&mut self) -> io::Result<crossterm::event::Event> {
            Ok(Receiver::recv(self)
                .map_err(|_| io::Error::new(io::ErrorKind::UnexpectedEof, "Input closed"))?
                .into())
        }

        fn read_timeout(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
            match Receiver::recv_timeout(self, timeout) {
                Ok(event) => Ok(Some(event.into())),
                Err(RecvTimeoutError::Timeout) => Ok(None),
                Err(RecvTimeoutError::Disconnected) => {
                    Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Input closed"))
                }
            }
        }
    }

    pub struct TestReader {
        tx: Sender<TestEvent>,
        rx: Receiver<TestEvent>,
    }

    impl TestReader {}

    enum TestEvent {}

    impl From<TestEvent> for crossterm::event::Event {
        fn from(value: TestEvent) -> Self {
            todo!()
        }
    }

    struct EventSender(Sender<TestEvent>);

    impl EventSender {
        pub fn send_keys(&self, seq: &str) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn parse_key_seq() {
        fn modifiers(s: &mut &str) -> ModalResult<KeyModifiers> {
            let modifier = dispatch! {take(2usize);
                "C-" => empty.value(KeyModifiers::CONTROL),
                "S-" => empty.value(KeyModifiers::SHIFT),
                "M-" => empty.value(KeyModifiers::META),
                "A-" => empty.value(KeyModifiers::ALT),
                _ => fail
            };
            repeat(1.., modifier)
                .fold(KeyModifiers::empty, BitOr::bitor)
                .parse_next(s)
        }

        fn char_code(s: &mut &str) -> ModalResult<KeyCode> {
            alt((preceded('\\', alt(('\\', '<', '>'))), none_of('\\')))
                .map(KeyCode::Char)
                .parse_next(s)
        }

        fn special_code(s: &mut &str) -> ModalResult<KeyCode> {
            alt(("ESC".value(KeyCode::Esc), "CR".value(KeyCode::Enter))).parse_next(s)
        }

        fn key_events(s: &mut &str) -> ModalResult<Vec<KeyEvent>> {
            repeat(
                0..,
                alt((
                    delimited('<', (modifiers, alt((special_code, char_code))), '>')
                        .map(|(modifiers, code)| KeyEvent::new(code, modifiers)),
                    alt((delimited('<', special_code, '>'), char_code))
                        .map(|code| KeyEvent::new(code, KeyModifiers::empty())),
                )),
            )
            .parse_next(s)
        }

        use KeyCode::{Char, Enter, Esc};
        // println!("{:#?}", key_events.parse("<ESC>").unwrap());
        pretty_assertions::assert_eq!(
            key_events
                .parse("<C-S-C><A-M-ESC>\\<ESC\\>\\\\<CR>")
                .unwrap(),
            vec![
                KeyEvent::new(Char('C'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
                KeyEvent::new(Esc, KeyModifiers::META | KeyModifiers::ALT),
                KeyEvent::new(Char('<'), KeyModifiers::empty()),
                KeyEvent::new(Char('E'), KeyModifiers::empty()),
                KeyEvent::new(Char('S'), KeyModifiers::empty()),
                KeyEvent::new(Char('C'), KeyModifiers::empty()),
                KeyEvent::new(Char('>'), KeyModifiers::empty()),
                KeyEvent::new(Char('\\'), KeyModifiers::empty()),
                KeyEvent::new(Enter, KeyModifiers::empty()),
            ]
        );
        dbg!();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::sync::Mutex;
//     use std::{io::Write, sync::Arc};
//
//     use crate::input::test::TestReader;
//     use crate::Reedline;
//
//     #[test]
//     fn feature() {
//         let w = TestWriter::default();
//         let reedline = Reedline::create_with(Box::new(TestReader::new()), Box::new(w.clone()));
//         reedline.read_line("ls");
//         println!("output {}", w.as_string());
//     }
//
//     #[derive(Clone, Default)]
//     struct TestWriter(Arc<Mutex<Vec<u8>>>);
//
//     impl TestWriter {
//         fn as_string(&self) -> String {
//             let guard = self.0.lock().unwrap();
//             String::from_utf8_lossy(&guard).to_string()
//         }
//     }
//
//     impl Write for TestWriter {
//         fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//             let mut guard = self.0.lock().unwrap();
//             guard.write(buf)
//         }
//
//         fn flush(&mut self) -> io::Result<()> {
//             let mut guard = self.0.lock().unwrap();
//             guard.flush()
//         }
//     }
// }
