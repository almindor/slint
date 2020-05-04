use std::convert::TryFrom;

#[repr(u16)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
pub enum SyntaxKind {
    Whitespace,
    Error,
    Eof,

    // Tokens:
    Identifier,
    Equal,
    LBrace,
    RBrace,
    Colon,

    //SyntaxKind:
    Document,
    Component,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(v: SyntaxKind) -> Self {
        rowan::SyntaxKind(v.into())
    }
}

#[derive(Clone, Debug)]
pub struct Token(SyntaxKind, rowan::SmolStr);

type ParseError = String;

pub struct Parser {
    builder: rowan::GreenNodeBuilder<'static>,
    tokens: Vec<Token>,
    cursor: usize,
    errors: Vec<ParseError>,
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Node<'a>(&'a mut Parser);
impl<'a> Drop for Node<'a> {
    fn drop(&mut self) {
        self.0.builder.finish_node();
    }
}

impl Parser {
    pub fn new(source: &str) -> Self {
        fn lex(source: &str) -> Vec<Token> {
            use SyntaxKind::*;
            fn tok(t: SyntaxKind) -> m_lexer::TokenKind {
                m_lexer::TokenKind(t.into())
            }
            let lexer = m_lexer::LexerBuilder::new()
                .error_token(tok(Error))
                .tokens(&[
                    (tok(Whitespace), r"\s+"),
                    (tok(Identifier), r"[\w]+"),
                    (tok(RBrace), r"\}"),
                    (tok(LBrace), r"\{"),
                    (tok(Equal), r"="),
                ])
                .build();
            lexer
                .tokenize(source)
                .into_iter()
                .scan(0usize, |start_offset, t| {
                    let s: rowan::SmolStr = source[*start_offset..*start_offset + t.len].into();
                    *start_offset += t.len;
                    Some(Token(SyntaxKind::try_from(t.kind.0).unwrap(), s))
                })
                .collect()
        }
        Self { builder: Default::default(), tokens: lex(source), cursor: 0, errors: vec![] }
    }

    pub fn start_node(&mut self, kind: SyntaxKind) -> Node {
        self.builder.start_node(kind.into());
        Node(self)
    }

    fn current_token(&self) -> Token {
        self.tokens.get(self.cursor).cloned().unwrap_or(Token(SyntaxKind::Eof, "".into()))
    }

    pub fn peek(&mut self) -> Token {
        self.consume_ws();
        self.current_token()
    }

    pub fn consume_ws(&mut self) {
        while self.current_token().0 == SyntaxKind::Whitespace {
            self.consume()
        }
    }

    pub fn consume(&mut self) {
        let t = self.current_token();
        self.builder.token(t.0.into(), t.1);
        self.cursor += 1;
    }

    pub fn expect(&mut self, kind: SyntaxKind) -> bool {
        let t = self.peek();
        if t.0 != kind {
            self.error("Syntax error"); // FIXME better error
            return false;
        }
        self.consume();
        return true;
    }

    pub fn error(&mut self, e: impl Into<String>) {
        self.errors.push(e.into());
    }
}

// Type = Base { /*...*/ }
fn parse_document(p: &mut Parser) -> bool {
    let mut p = p.start_node(SyntaxKind::Document);

    if !(p.expect(SyntaxKind::Identifier)
        && p.expect(SyntaxKind::Equal)
        && p.expect(SyntaxKind::Identifier)
        && p.expect(SyntaxKind::LBrace)
        && p.expect(SyntaxKind::RBrace))
    {
        return false;
    }

    p.expect(SyntaxKind::LBrace);
    if p.peek().0 != SyntaxKind::Eof {
        p.error("Should be end of file");
        return false;
    }
    return true;
}

pub fn parse(source: &str) -> (rowan::GreenNode, Vec<ParseError>) {
    let mut p = Parser::new(source);
    parse_document(&mut p);
    (p.builder.finish(), p.errors)
}
