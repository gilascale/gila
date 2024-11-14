use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Position {
    index: u32, // 0-based
    line: u32,  // 0-based
}

#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    RETURN,
    LPAREN,
    RPAREN,
    ASSIGN,
    EQUALS,
    PASS,
    FN,
    DO,
    THEN,
    END,
    ADD,
    SUB,
    MUL,
    DIV,
    NUMBER(Rc<String>),
    IDENTIFIER(Rc<String>),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub pos: Position,
    pub typ: Type,
}

#[derive(Debug)]
pub struct Lexer {}

impl Lexer {
    pub fn lex(&self, source: std::string::String) -> std::vec::Vec<Token> {
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
                '+' => {
                    v.push(Token {
                        typ: Type::ADD,
                        pos: Position { index, line },
                    });
                }
                '-' => {
                    v.push(Token {
                        typ: Type::SUB,
                        pos: Position { index, line },
                    });
                }
                '*' => {
                    v.push(Token {
                        typ: Type::MUL,
                        pos: Position { index, line },
                    });
                }
                '/' => {
                    v.push(Token {
                        typ: Type::DIV,
                        pos: Position { index, line },
                    });
                }
                '.' => {
                    if chars[counter + 1] == '.' && chars[counter + 2] == '.' {
                        counter += 2;
                        index += 2;
                    }
                }
                'd' => {
                    if chars[counter + 1] == 'o' {
                        v.push(Token {
                            typ: Type::DO,
                            pos: Position { index, line },
                        });
                        counter += 1;
                        index += 1;
                    }
                }
                'f' => {
                    if chars[counter + 1] == 'n' {
                        v.push(Token {
                            typ: Type::FN,
                            pos: Position { index, line },
                        });
                        counter += 1;
                        index += 1;
                    }
                }
                'e' => {
                    if chars[counter + 1] == 'n' && chars[counter + 2] == 'd' {
                        v.push(Token {
                            typ: Type::END,
                            pos: Position { index, line },
                        });
                        counter += 2;
                        index += 2;
                    }
                }
                't' => {
                    if chars[counter + 1] == 'h'
                        && chars[counter + 2] == 'e'
                        && chars[counter + 3] == 'n'
                    {
                        v.push(Token {
                            typ: Type::THEN,
                            pos: Position { index, line },
                        });
                        counter += 3;
                        index += 3;
                    }
                }
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
                'r' => {
                    if chars[counter + 1] == 'e'
                        && chars[counter + 2] == 't'
                        && chars[counter + 3] == 'u'
                        && chars[counter + 4] == 'r'
                        && chars[counter + 5] == 'n'
                    {
                        v.push(Token {
                            typ: Type::RETURN,
                            pos: Position { index, line },
                        });
                        counter += 5;
                        index += 5;
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
                    if chars[counter + 1] == '=' {
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
                            typ: Type::IDENTIFIER(identifier.into()),
                            pos: Position { index, line },
                        });
                        continue;
                    }
                    if current.is_numeric() {
                        let mut identifier = "".to_string();
                        while counter < chars.len() {
                            if chars[counter].is_whitespace() || !chars[counter].is_numeric() {
                                break;
                            }
                            let next = chars[counter];
                            identifier.push(next);
                            index += 1;
                            counter += 1;
                        }
                        // identifier
                        v.push(Token {
                            typ: Type::NUMBER(identifier.into()),
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
