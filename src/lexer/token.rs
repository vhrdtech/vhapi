use std::{fmt, ops};
use nom_locate::{LocatedSpan};
#[cfg(feature = "trace")]
use nom_tracable::{TracableInfo, HasTracableInfo};
use nom_greedyerror::{GreedyError, Position};
use std::fmt::Formatter;
use nom_packrat::HasExtraState;
use nom::{InputLength, InputTake, Slice, InputIter, Compare, CompareResult};
use std::ops::{Range, RangeTo, RangeFrom, RangeFull};
use std::iter::Enumerate;
use crate::lexer::token::LitKind::Byte;

/// Bool / Byte / Char / Integer / Float / Str / ByteStr / Err
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LitKind {
    Bool,
    Byte,
    Char,
    Integer,
    Float,
    Str,
    //StrRaw(u16),
    ByteStr,
    //ByteStrRaw(u16),
    Err
}

/// LitKind + Symbol + Optional suffix
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lit {
    pub kind: LitKind,
    //pub symbol: Symbol,
    //pub suffix: Option<Symbol>
}

/// Reserved or normal identifier
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IdentToken {
    Let,
    Fn,
    If,
    Else,
    Return,
    Normal
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenKind {
    // Expression operators
    /// "="
    Assign,

    // Unary op tokens
    /// "~"
    Tilde,
    /// "!"
    Excl,

    // Bool op tokens
    /// "<"
    Lt,
    /// "<="
    Le,
    /// "=="
    EqEq,
    /// "!="
    Ne,
    /// ">"
    Gt,
    /// ">="
    Ge,
    /// "&&"
    AndAnd,
    /// "||"
    OrOr,

    // Binary op tokens
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `^`
    Caret,
    /// `&`
    And,
    /// `|`
    Or,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    //?BinOpEq(BinOpToken),

    // Structural symbols
    /// "@"
    At,
    /// "."
    Dot,
    /// ".."
    DotDot,
    /// ","
    Comma,
    /// ";"
    Semicolon,
    /// ":"
    Colon,
    /// "<" as arrow
    RArrow,
    /// ">" as arrow
    LArrow,
    //FatArrow,
    /// "#"
    Pound,
    /// "$"
    Dollar,
    /// "?"
    Question,

    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,

    // Literals
    Literal(Lit),

    Ident(IdentToken),

    /// Any whitespace
    Whitespace,
    Comment,

    Unkown(/*name*/),

    Eof,
}

// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord)]
// pub enum LiteralKind {
//     /// "127_u8", "0o100", "0b129i99"
//     Int { base: Base },
//     /// "12.34f32", "56f16"
//     Float { base: Base },
//     /// "'a'", "'\\'", "'''"
//     Char,
//     /// "b'a'", "b'\\'", "b'''"
//     //Byte,
//     /// ""abc""
//     Str,
//     /// "b"abc""
//     //ByteStr,
//     /// "r"abc""
//     //RawStr,
//     /// "br"abc""
//     //RawByteStr
// }
//
// /// Base of numeric literal
// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord)]
// pub enum Base {
//     /// 0b prefix
//     Binary,
//     /// 0o prefix
//     Octal,
//     /// 0x prefix
//     Hexadecimal,
//     /// Without prefix
//     Decimal
// }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BytePos(pub u32);

/// Token span [lo, hi)
#[derive(Clone, Copy, PartialEq)]
pub struct Span {
    lo: BytePos,
    hi: BytePos
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.hi.0 >= self.lo.0 {
            write!(f, "bytes[{}..{})", self.lo.0, self.hi.0)
        } else {
            write!(f, "any")
        }
    }
}

impl Span {
    /// Construct invalid Span that is equal to any other Span (when comparing TokenStream's).
    pub fn any() -> Self {
        Span {
            lo: BytePos(1u32),
            hi: BytePos(0u32)
        }
    }
}

impl ops::Add<Span> for Span {
    type Output = Span;

    fn add(self, rhs: Span) -> Span {
        Span {
            lo: BytePos(self.lo.0),
            hi: BytePos(rhs.hi.0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub(crate) struct NLSpanInfo {
    #[cfg(feature = "trace")]
    pub traceable_info: TracableInfo,
    //pub recursive_info: RecursiveInfo
}

impl NLSpanInfo {
    #[cfg(feature = "trace")]
    pub fn new() -> Self {
        NLSpanInfo {
            traceable_info: TracableInfo::new()
        }
    }
    #[cfg(not(feature = "trace"))]
    pub fn new() -> Self {
        NLSpanInfo { }
    }
}

pub(crate) type NLSpan<'a> = LocatedSpan<&'a str, NLSpanInfo>;
pub(crate) type IResult<T, U> = nom::IResult<T, U, GreedyError<T>>;

// impl HasRecursiveInfo for NLSpanInfo {
//     fn get_recursive_info(&self) -> RecursiveInfo {
//         self.recursive_info
//     }
//
//     fn set_recursive_info(mut self, info: RecursiveInfo) -> Self {
//         self.recursive_info = info;
//         self
//     }
// }

#[cfg(feature = "trace")]
impl HasTracableInfo for NLSpanInfo {
    fn get_tracable_info(&self) -> TracableInfo {
        self.traceable_info
    }

    fn set_tracable_info(mut self, info: TracableInfo) -> Self {
        self.traceable_info = info;
        self
    }
}

impl HasExtraState<()> for NLSpanInfo {
    fn get_extra_state(&self) -> () {
        ()
    }
}

impl<'a> From<NLSpan<'a>> for Span {
    fn from(nlspan: NLSpan<'a>) -> Self {
        let pos = nlspan.location_offset() as u32;
        let len = nlspan.fragment().len() as u32;
        Span {
            lo: BytePos(pos),
            hi: BytePos(pos + len)
        }
    }
}

nom_packrat::storage!(Token);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TokenStream<'a> {
    pub toks: &'a [Token],
    pub start: usize,
    pub end: usize
}

impl<'a> TokenStream<'a> {
    pub fn new(vec: &'a Vec<Token>) -> Self {
        TokenStream {
            toks: vec.as_slice(),
            start: 0,
            end: vec.len()
        }
    }

    pub fn new_with_slice(slice: &'a [Token]) -> Self {
        TokenStream {
            toks: slice,
            start: 0,
            end: slice.len()
        }
    }
}

impl<'a, 'b> Compare<TokenStream<'b>> for TokenStream<'a> {
    fn compare(&self, t: TokenStream<'b>) -> CompareResult {
        let pos = self.iter_elements().zip(
            t.iter_elements()).position(
            |(a, b)| a.kind != b.kind);

        match pos {
            Some(_) => CompareResult::Error,
            None => {
                if self.input_len() >= t.input_len() {
                    CompareResult::Ok
                } else {
                    CompareResult::Incomplete
                }
            }
        }
    }

    fn compare_no_case(&self, t: TokenStream<'b>) -> CompareResult {
        CompareResult::Ok
    }
}

impl<'a> InputLength for TokenStream<'a> {
    fn input_len(&self) -> usize {
        self.toks.len()
    }
}

impl<'a> InputTake for TokenStream<'a> {
    fn take(&self, count: usize) -> Self {
        TokenStream {
            toks: &self.toks[0..count],
            start: 0,
            end: count
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.toks.split_at(count);
        let first = TokenStream {
            toks: prefix,
            start: 0,
            end: prefix.len()
        };
        let second = TokenStream {
            toks: suffix,
            start: 0,
            end: suffix.len()
        };
        (second, first)
    }
}

impl InputLength for Token {
    fn input_len(&self) -> usize {
        1
    }
}

impl<'a> Slice<Range<usize>> for TokenStream<'a> {
    fn slice(&self, range: Range<usize>) -> Self {
        TokenStream {
            toks: &self.toks[range.clone()],
            start: self.start + range.start,
            end: self.start + range.end
        }
    }
}

impl<'a> Slice<RangeTo<usize>> for TokenStream<'a> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.slice(0..range.end)
    }
}

impl<'a> Slice<RangeFrom<usize>> for TokenStream<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.slice(range.start..self.end - self.start)
    }
}

impl<'a> Slice<RangeFull> for TokenStream<'a> {
    fn slice(&self, _: RangeFull) -> Self {
        TokenStream {
            toks: self.toks,
            start: self.start,
            end: self.end
        }
    }
}

impl<'a> InputIter for TokenStream<'a> {
    type Item = &'a Token;
    type Iter = Enumerate<::std::slice::Iter<'a, Token>>;
    type IterElem = ::std::slice::Iter<'a, Token>;

    fn iter_indices(&self) -> Self::Iter {
        self.toks.iter().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.toks.iter()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.toks.iter().position(|b| predicate(b))
    }

    fn slice_index(&self, count: usize) -> Option<usize> {
        if self.toks.len() >= count {
            Some(count)
        } else {
            None
        }
    }
}