use std::iter::Peekable;

use crate::{edit_mode::vi::ViMode, EditCommand, ReedlineEvent, Vi};

use super::parser::{ParseResult, ReedlineOption};

pub fn parse_text_object<'iter, I>(
    input: &mut Peekable<I>,
    command_char: Option<char>,
) -> ParseResult<TextObject>
where
    I: Iterator<Item = &'iter char>,
{
    match input.peek() {
        Some('w') => {
            let _ = input.next();
            ParseResult::Valid(TextObject::Word)
        }
        None => ParseResult::Incomplete,
        _ => ParseResult::Invalid,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TextObject {
    Word,
    BigWord,
    // Sentence,
    // Paragraph,
    Parenthesis,
    Bracket,
    CurlyBrace,
    Caret,
    DoubleQuote,
    SingleQuote,
    BackTick,
    // Tag,
}

impl TextObject {
    pub fn to_reedline(&self, vi_state: &mut Vi) -> Vec<ReedlineOption> {
        let select_mode = vi_state.mode == ViMode::Visual;
        todo!()
        //     match self {
        //         TextObject::Left => vec![ReedlineOption::Event(ReedlineEvent::UntilFound(vec![
        //             ReedlineEvent::MenuLeft,
        //             ReedlineEvent::Edit(vec![EditCommand::MoveLeft {
        //                 select: select_mode,
        //             }]),
        //         ]))],
        //    }
    }
}
