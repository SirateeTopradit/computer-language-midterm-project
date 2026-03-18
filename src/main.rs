use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::iter::Peekable;
use std::str::Chars;

/// Lexical Tokens for SRISC
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Code,
    End,
    AluOp(String),
    MemOp(String),
    BrOp(String),
    Reg(String),
    Imm(i32),
    Label(String),
    Colon,
    Comma,
    LParen,
    RParen,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Code => write!(f, ".code"),
            Token::End => write!(f, ".end"),
            Token::AluOp(op) => write!(f, "{}", op),
            Token::MemOp(op) => write!(f, "{}", op),
            Token::BrOp(op) => write!(f, "{}", op),
            Token::Reg(reg) => write!(f, "{}", reg),
            Token::Imm(imm) => write!(f, "{}", imm),
            Token::Label(label) => write!(f, "{}", label),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Errors occurring during lexical analysis
#[derive(Debug)]
pub enum LexerError {
    InvalidCharacter(char),
    InvalidLabel(String),
    InvalidRegister(String),
    InvalidNumber(String),
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::InvalidCharacter(c) => write!(f, "Invalid character: {}", c),
            LexerError::InvalidLabel(s) => write!(f, "Invalid label: {}", s),
            LexerError::InvalidRegister(s) => write!(f, "Invalid register: {}", s),
            LexerError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
        }
    }
}

impl Error for LexerError {}

/// The Lexical Analyzer (Lexer)
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else if c == ';' {
                // Skip comment to end of line
                while let Some(c) = self.chars.next() {
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '.' || c == '_' {
                ident.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }
        ident
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace_and_comments();

        let c = match self.chars.next() {
            Some(c) => c,
            None => return None,
        };

        match c {
            ':' => Some(Ok(Token::Colon)),
            ',' => Some(Ok(Token::Comma)),
            '(' => Some(Ok(Token::LParen)),
            ')' => Some(Ok(Token::RParen)),
            '.' => {
                let mut ident = String::from(".");
                ident.push_str(&self.read_identifier());
                match ident.as_str() {
                    ".code" => Some(Ok(Token::Code)),
                    ".end" => Some(Ok(Token::End)),
                    _ => Some(Err(LexerError::InvalidCharacter('.'))),
                }
            }
            c if c.is_ascii_alphabetic() => {
                let mut ident = String::from(c);
                ident.push_str(&self.read_identifier());
                match ident.as_str() {
                    "add" | "sub" | "and" | "or" => Some(Ok(Token::AluOp(ident))),
                    "ld" | "sd" | "lw" | "sw" => Some(Ok(Token::MemOp(ident))),
                    "beq" | "bne" | "blt" | "bge" => Some(Ok(Token::BrOp(ident))),
                    s if s.starts_with('x') => {
                        let reg_num = &s[1..];
                        if let Ok(n) = reg_num.parse::<u32>() {
                            if n <= 31 {
                                return Some(Ok(Token::Reg(s.to_string())));
                            }
                        }
                        Some(Err(LexerError::InvalidRegister(s.to_string())))
                    }
                    s if s.starts_with('L') => {
                        let label_num = &s[1..];
                        if let Ok(n) = label_num.parse::<u32>() {
                            if n <= 10 {
                                return Some(Ok(Token::Label(s.to_string())));
                            }
                        }
                        Some(Err(LexerError::InvalidLabel(s.to_string())))
                    }
                    _ => Some(Err(LexerError::InvalidCharacter(c))),
                }
            }
            c if c.is_ascii_digit() || c == '-' => {
                let mut num_str = String::from(c);
                while let Some(&nc) = self.chars.peek() {
                    if nc.is_ascii_digit() {
                        num_str.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match num_str.parse::<i32>() {
                    Ok(n) => Some(Ok(Token::Imm(n))),
                    Err(_) => Some(Err(LexerError::InvalidNumber(num_str))),
                }
            }
            c => Some(Err(LexerError::InvalidCharacter(c))),
        }
    }
}

/// Errors occurring during parsing
#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { expected: String, found: Token },
    LexerError(LexerError),
    EarlyEof,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            ParseError::LexerError(e) => write!(f, "Lexical error: {}", e),
            ParseError::EarlyEof => write!(f, "Unexpected end of file"),
        }
    }
}

impl Error for ParseError {}

/// The Syntax Analyzer (Parser)
pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Parser {
            lexer: lexer.peekable(),
        }
    }

    fn advance(&mut self) -> Result<Token, ParseError> {
        match self.lexer.next() {
            Some(Ok(token)) => Ok(token),
            Some(Err(e)) => Err(ParseError::LexerError(e)),
            None => Ok(Token::Eof),
        }
    }

    fn peek(&mut self) -> Result<Token, ParseError> {
        match self.lexer.peek() {
            Some(Ok(token)) => Ok(token.clone()),
            Some(Err(_)) => {
                // We need to consume the error to return it
                match self.lexer.next().unwrap() {
                    Err(e) => Err(ParseError::LexerError(e)),
                    _ => unreachable!(),
                }
            }
            None => Ok(Token::Eof),
        }
    }

    fn match_token(&mut self, expected_desc: &str, predicate: impl FnOnce(&Token) -> bool) -> Result<Token, ParseError> {
        let token = self.peek()?;
        if predicate(&token) {
            self.advance()
        } else {
            Err(ParseError::UnexpectedToken {
                expected: expected_desc.to_string(),
                found: token,
            })
        }
    }

    /// Program -> CODE StmtList END
    pub fn parse_program(&mut self) -> Result<(), ParseError> {
        self.match_token(".code", |t| matches!(t, Token::Code))?;
        self.parse_stmt_list()?;
        self.match_token(".end", |t| matches!(t, Token::End))?;
        
        let token = self.peek()?;
        if token != Token::Eof {
             return Err(ParseError::UnexpectedToken {
                expected: "EOF".to_string(),
                found: token,
            });
        }
        Ok(())
    }

    /// StmtList -> Stmt StmtList | epsilon
    fn parse_stmt_list(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        match token {
            Token::Label(_) | Token::AluOp(_) | Token::MemOp(_) | Token::BrOp(_) => {
                self.parse_stmt()?;
                self.parse_stmt_list()
            }
            Token::End => Ok(()), // epsilon
            _ => Err(ParseError::UnexpectedToken {
                expected: "statement or .end".to_string(),
                found: token,
            }),
        }
    }

    /// Stmt -> LABEL COLON | Instruction
    fn parse_stmt(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        match token {
            Token::Label(_) => {
                self.advance()?;
                self.match_token(":", |t| matches!(t, Token::Colon))?;
                Ok(())
            }
            Token::AluOp(_) | Token::MemOp(_) | Token::BrOp(_) => {
                self.parse_instruction()
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "label or instruction".to_string(),
                found: token,
            }),
        }
    }

    /// Instruction -> ALU_OP REG COMMA REG COMMA Operand
    ///              | MEM_OP REG COMMA IMM LPAREN REG RPAREN
    ///              | BR_OP REG COMMA REG COMMA LABEL
    fn parse_instruction(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        match token {
            Token::AluOp(_) => {
                self.advance()?;
                self.match_token("register", |t| matches!(t, Token::Reg(_)))?;
                self.match_token(",", |t| matches!(t, Token::Comma))?;
                self.match_token("register", |t| matches!(t, Token::Reg(_)))?;
                self.match_token(",", |t| matches!(t, Token::Comma))?;
                self.parse_operand()?;
                Ok(())
            }
            Token::MemOp(_) => {
                self.advance()?;
                self.match_token("register", |t| matches!(t, Token::Reg(_)))?;
                self.match_token(",", |t| matches!(t, Token::Comma))?;
                self.match_token("immediate", |t| matches!(t, Token::Imm(_)))?;
                self.match_token("(", |t| matches!(t, Token::LParen))?;
                self.match_token("register", |t| matches!(t, Token::Reg(_)))?;
                self.match_token(")", |t| matches!(t, Token::RParen))?;
                Ok(())
            }
            Token::BrOp(_) => {
                self.advance()?;
                self.match_token("register", |t| matches!(t, Token::Reg(_)))?;
                self.match_token(",", |t| matches!(t, Token::Comma))?;
                self.match_token("register", |t| matches!(t, Token::Reg(_)))?;
                self.match_token(",", |t| matches!(t, Token::Comma))?;
                self.match_token("label", |t| matches!(t, Token::Label(_)))?;
                Ok(())
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "instruction".to_string(),
                found: token,
            }),
        }
    }

    /// Operand -> REG | IMM
    fn parse_operand(&mut self) -> Result<(), ParseError> {
        let token = self.peek()?;
        match token {
            Token::Reg(_) | Token::Imm(_) => {
                self.advance()?;
                Ok(())
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "register or immediate".to_string(),
                found: token,
            }),
        }
    }
}

fn main() {
    let mut input = String::new();
    if let Err(_) = io::stdin().read_to_string(&mut input) {
        println!("no");
        return;
    }

    let lexer = Lexer::new(&input);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(_) => println!("yes"),
        Err(_) => println!("no"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid(input: &str) -> bool {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        parser.parse_program().is_ok()
    }

    #[test]
    fn test_valid_arithmetic() {
        assert!(is_valid(".code add x1, x2, x3 .end"));
        assert!(is_valid(".code add x1, x2, 10 .end"));
    }

    #[test]
    fn test_valid_memory() {
        assert!(is_valid(".code ld x1, 10(x2) .end"));
        assert!(is_valid(".code sw x31, -4(x0) .end"));
    }

    #[test]
    fn test_valid_branch() {
        assert!(is_valid(".code beq x1, x2, L0 .end"));
    }

    #[test]
    fn test_valid_loop_example() {
        let input = "
            .code
            L0:
                add x1, x1, 1
                blt x1, x10, L0
            .end
        ";
        assert!(is_valid(input));
    }

    #[test]
    fn test_invalid_syntax_missing_comma() {
        assert!(!is_valid(".code add x1 x2, x3 .end"));
    }

    #[test]
    fn test_invalid_lexical_token() {
        assert!(!is_valid(".code add x1, x2, x32 .end")); // x32 is invalid
        assert!(!is_valid(".code add x1, x2, L11 .end")); // L11 is invalid
        assert!(!is_valid(".code @ .end")); // @ is invalid
    }
    
    #[test]
    fn test_comments() {
        let input = "
            .code
            add x1, x2, x3 ; this is a comment
            .end ; end of code
        ";
        assert!(is_valid(input));
    }
}
