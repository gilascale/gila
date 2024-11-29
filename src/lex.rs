use std::rc::Rc;

use deepsize::DeepSizeOf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub index: u32, // 0-based
    pub line: u32,  // 0-based
    pub index_end: u32,
    pub line_end: u32, // todo add end
}

impl Position {
    pub fn join(&self, other: Position) -> Position {
        // todo
        return Position {
            index: self.index,
            line: self.line,
            index_end: other.index_end,
            line_end: other.line_end,
        };
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash, DeepSizeOf)]
pub enum Type {
    RETURN,
    LPAREN,
    RPAREN,
    ASSIGN,
    EQUALS,
    PASS,
    FN,
    IF,
    DO,
    THEN,
    TYPE,
    LET,
    END,
    ADD,
    SUB,
    MUL,
    DIV,
    COLON,
    U32,
    NUMBER(Rc<String>),
    ATOM(Rc<String>),
    IDENTIFIER(Rc<String>),
    STRING(Rc<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        // let mut position: Position = Position { index: 0, line: 0 };

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
                        pos: Position {
                            index,
                            line,
                            index_end: index + 1,
                            line_end: line,
                        },
                    });
                }
                ':' => {
                    v.push(Token {
                        typ: Type::COLON,
                        pos: Position {
                            index,
                            line,
                            index_end: index + 1,
                            line_end: line,
                        },
                    });
                }
                '-' => {
                    v.push(Token {
                        typ: Type::SUB,
                        pos: Position {
                            index,
                            line,
                            index_end: index + 1,
                            line_end: line,
                        },
                    });
                }
                '*' => {
                    v.push(Token {
                        typ: Type::MUL,
                        pos: Position {
                            index,
                            line,
                            index_end: index + 1,
                            line_end: line,
                        },
                    });
                }
                '/' => {
                    v.push(Token {
                        typ: Type::DIV,
                        pos: Position {
                            index,
                            line,
                            index_end: index + 1,
                            line_end: line,
                        },
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
                            pos: Position {
                                index,
                                line,
                                index_end: index + 2,
                                line_end: line,
                            },
                        });
                        counter += 1;
                        index += 1;
                    }
                }
                'f' => {
                    if chars[counter + 1] == 'n' {
                        v.push(Token {
                            typ: Type::FN,
                            pos: Position {
                                index,
                                line,
                                index_end: index + 2,
                                line_end: line,
                            },
                        });
                        counter += 1;
                        index += 1;
                    }
                }
                'i' => {
                    if chars[counter + 1] == 'f' {
                        v.push(Token {
                            typ: Type::IF,
                            pos: Position {
                                index,
                                line,
                                index_end: index + 2,
                                line_end: line,
                            },
                        });
                        counter += 1;
                        index += 1;
                    }
                }
                'e' => {
                    if chars[counter + 1] == 'n' && chars[counter + 2] == 'd' {
                        v.push(Token {
                            typ: Type::END,
                            pos: Position {
                                index,
                                line,
                                index_end: index + 3,
                                line_end: line,
                            },
                        });
                        counter += 2;
                        index += 2;
                    }
                }
                'l' => {
                    if chars[counter + 1] == 'e' && chars[counter + 2] == 't' {
                        v.push(Token {
                            typ: Type::LET,
                            pos: Position {
                                index,
                                line,
                                index_end: index + 3,
                                line_end: line,
                            },
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
                            pos: Position {
                                index,
                                line,
                                index_end: index + 4,
                                line_end: line,
                            },
                        });
                        counter += 3;
                        index += 3;
                    } else if chars[counter + 1] == 'y'
                        && chars[counter + 2] == 'p'
                        && chars[counter + 3] == 'e'
                    {
                        v.push(Token {
                            typ: Type::TYPE,
                            pos: Position {
                                index,
                                line,
                                index_end: index + 4,
                                line_end: line,
                            },
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
                            pos: Position {
                                index,
                                line,
                                index_end: index + 4,
                                line_end: line,
                            },
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
                            pos: Position {
                                index,
                                line,
                                index_end: index + 6,
                                line_end: line,
                            },
                        });
                        counter += 5;
                        index += 5;
                    }
                }
                'u' => {
                    if chars[counter + 1] == '3' && chars[counter + 2] == '2' {
                        v.push(Token {
                            typ: Type::U32,
                            pos: Position {
                                index,
                                line,
                                index_end: index + 3,
                                line_end: line,
                            },
                        });
                        counter += 2;
                        index += 2;
                    }
                }
                '(' => v.push(Token {
                    typ: Type::LPAREN,
                    pos: Position {
                        index,
                        line,
                        index_end: index + 1,
                        line_end: line,
                    },
                }),
                ')' => v.push(Token {
                    typ: Type::RPAREN,
                    pos: Position {
                        index,
                        line,
                        index_end: index + 1,
                        line_end: line,
                    },
                }),
                '=' => {
                    if chars[counter + 1] == '=' {
                        v.push(Token {
                            typ: Type::EQUALS,
                            pos: Position {
                                index,
                                line,
                                index_end: index + 2,
                                line_end: line,
                            },
                        });
                        index += 1;
                        counter += 1;
                    } else {
                        v.push(Token {
                            typ: Type::ASSIGN,
                            pos: Position {
                                index,
                                line,
                                index_end: index + 1,
                                line_end: line,
                            },
                        })
                    }
                }
                '"' => {
                    // fixme deal with multiline
                    let tmp_index = index;
                    let mut s = "".to_string();
                    index += 1;
                    counter += 1;

                    while counter < chars.len() {
                        let next = chars[counter];
                        if next == '"' {
                            counter += 1;
                            index += 1;
                            break;
                        };
                        s.push(next);
                        index += 1;
                        counter += 1;
                    }
                    v.push(Token {
                        typ: Type::STRING(s.into()),
                        pos: Position {
                            index: tmp_index,
                            line: line,
                            index_end: index,
                            line_end: line,
                        },
                    });
                }
                ':' => {
                    index += 1;
                    counter += 1;
                    if current.is_alphabetic() {
                        let tmp_index = index;
                        let mut identifier = "".to_string();
                        while counter < chars.len() {
                            if !chars[counter].is_alphabetic() {
                                break;
                            }
                            let next = chars[counter];
                            identifier.push(next);
                            index += 1;
                            counter += 1;
                        }
                        // identifier
                        v.push(Token {
                            typ: Type::ATOM(identifier.into()),
                            pos: Position {
                                index: tmp_index,
                                line,
                                index_end: index + 1,
                                line_end: line,
                            },
                        });
                        continue;
                    }
                }
                _ => {
                    if current.is_alphabetic() {
                        let tmp_index = index;
                        let mut identifier = "".to_string();
                        while counter < chars.len() {
                            if !chars[counter].is_alphabetic() {
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
                            pos: Position {
                                index: tmp_index,
                                line,
                                index_end: index + 1,
                                line_end: line,
                            },
                        });
                        continue;
                    }
                    if current.is_numeric() {
                        let mut identifier = "".to_string();
                        let tmp_index = index;
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
                            pos: Position {
                                index: tmp_index,
                                line,
                                index_end: index + 1,
                                line_end: line,
                            },
                        });
                        continue;
                    }
                }
            }

            index += 1;
            counter += 1;
        }

        return v;
    }
}
