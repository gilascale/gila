#[derive(Debug)]
pub struct Position {
    index: u32, // 0-based
    line: u32,  // 0-based
}

#[derive(Debug)]
pub enum Type {
    LPAREN,
    RPAREN,
    ASSIGN,
    EQUALS,
    PASS,
    NUMBER(String),
    IDENTIFIER(String),
}

#[derive(Debug)]
pub struct Token {
    pos: Position,
    typ: Type,
}

#[derive(Debug)]
pub struct Lexer {}

impl Lexer {
    pub fn lex(&self, source: &str) -> std::vec::Vec<Token> {
        let mut v = vec![];
        let mut counter = 0;
        let mut index = 0;
        let mut line = 0;
        let mut position = Position { index: 0, line: 0 };

        let chars: Vec<char> = source.chars().collect();

        while counter < chars.len() {
            let current = chars[counter];

            if current.is_whitespace() {
                if current == '\n' {
                    line += 1;
                    index = 0;
                }
                counter += 1;
                continue;
            }

            // potential identifier?

            match current {
                'p' => {
                    if chars[counter + 1] == 'a'
                        && chars[counter + 2] == 's'
                        && chars[counter + 3] == 's'
                    {
                        v.push(Token {
                            typ: Type::PASS,
                            pos: Position { index, line },
                        });
                        counter += 3;
                        index += 3;
                    }
                }
                '(' => v.push(Token {
                    typ: Type::LPAREN,
                    pos: Position { index, line },
                }),
                ')' => v.push(Token {
                    typ: Type::RPAREN,
                    pos: Position { index, line },
                }),
                '=' => {
                    if (chars[counter + 1] == '=') {
                        v.push(Token {
                            typ: Type::EQUALS,
                            pos: Position { index, line },
                        });
                        index += 1;
                        counter += 1;
                    } else {
                        v.push(Token {
                            typ: Type::ASSIGN,
                            pos: Position { index, line },
                        })
                    }
                }
                _ => {
                    if current.is_alphabetic() {
                        let mut identifier = "".to_string();
                        while counter < chars.len() {
                            if chars[counter].is_whitespace() {
                                break;
                            }
                            let next = chars[counter];
                            identifier.push(next);
                            index += 1;
                            counter += 1;
                        }
                        // identifier
                        v.push(Token {
                            typ: Type::IDENTIFIER(identifier),
                            pos: Position { index, line },
                        });
                        continue;
                    }
                }
            }

            index += 1;
            counter += 1;
        }

        println!("lexing {:?}... done!", source);

        return v;
    }
}
