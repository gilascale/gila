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
    DOLLAR,
    ASSERT,
    EXCLAIM,
    TEST,
    AMPERSAND,
    DOT,
    RETURN,
    LPAREN,
    RPAREN,
    LSQUARE,
    RSQUARE,
    GREATER_THAN,
    LESS_THAN,
    GREATER_EQ,
    LESS_EQ,
    OR,
    AND,
    COMMA,
    ASSIGN,
    EQUALS,
    NOT_EQUALS,
    PASS,
    TRUE,
    FALSE,
    FN,
    FOR,
    IF,
    IN,
    ELSE,
    DO,
    THEN,
    TYPE,
    LET,
    IMPORT,
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
pub struct Lexer {
    pub counter: u32,
    pub index: u32,
    pub line: u32,
}

impl Lexer {
    pub fn new() -> Self {
        return Lexer {
            counter: 0,
            index: 0,
            line: 0,
        };
    }

    pub fn lex(&mut self, source: std::string::String) -> std::vec::Vec<Token> {
        let mut v = vec![];
        // let mut position: Position = Position { index: 0, line: 0 };

        let chars: Vec<char> = source.chars().collect();

        while self.counter < chars.len().try_into().unwrap() {
            let current = chars[self.counter as usize];

            if current.is_whitespace() {
                if current == '\n' {
                    self.line += 1;
                    self.index = 0;
                }
                self.counter += 1;
                continue;
            }

            // potential identifier?

            match current {
                '$' => {
                    v.push(Token {
                        typ: Type::DOLLAR,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }
                '@' => {
                    v.push(Token {
                        typ: Type::AMPERSAND,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }
                '.' => {
                    v.push(Token {
                        typ: Type::DOT,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }
                '>' => {
                    if chars[self.counter as usize + 1] == '=' {
                        v.push(Token {
                            typ: Type::GREATER_EQ,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        });
                        self.counter += 1;
                        self.index += 1;
                    } else {
                        v.push(Token {
                            typ: Type::GREATER_THAN,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        });
                    }
                }
                '<' => {
                    if chars[self.counter as usize + 1] == '=' {
                        v.push(Token {
                            typ: Type::LESS_EQ,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        });
                        self.counter += 1;
                        self.index += 1;
                    } else {
                        v.push(Token {
                            typ: Type::LESS_THAN,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        });
                    }
                }
                '[' => {
                    v.push(Token {
                        typ: Type::LSQUARE,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }

                ']' => {
                    v.push(Token {
                        typ: Type::RSQUARE,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }
                '+' => {
                    v.push(Token {
                        typ: Type::ADD,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }
                ':' => {
                    v.push(Token {
                        typ: Type::COLON,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }
                '-' => {
                    v.push(Token {
                        typ: Type::SUB,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }
                '*' => {
                    v.push(Token {
                        typ: Type::MUL,
                        pos: Position {
                            index: self.index,
                            line: self.line,
                            index_end: self.index + 1,
                            line_end: self.line,
                        },
                    });
                }
                '/' => {
                    // todo cant have comments without a \n at the end
                    if chars[self.counter as usize + 1] == '/' {
                        while chars[self.counter as usize + 1] != '\n' {
                            self.counter += 1;
                        }
                        self.counter += 1;
                    } else {
                        v.push(Token {
                            typ: Type::DIV,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        });
                    }
                }
                '.' => {
                    if chars[self.counter as usize + 1] == '.'
                        && chars[self.counter as usize + 2] == '.'
                    {
                        self.counter += 2;
                        self.index += 2;
                    }
                }
                'o' => {
                    if chars[self.counter as usize + 1] == 'r'
                    // todo implement this end check
                        && !chars[self.counter as usize + 2].is_alphabetic()
                    {
                        v.push(Token {
                            typ: Type::OR,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 2,
                                line_end: self.line,
                            },
                        });
                        self.counter += 1;
                        self.index += 1;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                'd' => {
                    if chars[self.counter as usize + 1] == 'o'
                    // todo implement this end check
                        && !chars[self.counter as usize + 2].is_alphabetic()
                    {
                        v.push(Token {
                            typ: Type::DO,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 2,
                                line_end: self.line,
                            },
                        });
                        self.counter += 1;
                        self.index += 1;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }

                'f' => {
                    if chars[self.counter as usize + 1] == 'n' {
                        v.push(Token {
                            typ: Type::FN,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 2,
                                line_end: self.line,
                            },
                        });
                        self.counter += 1;
                        self.index += 1;
                    } else if chars[self.counter as usize + 1] == 'a'
                        && chars[self.counter as usize + 2] == 'l'
                        && chars[self.counter as usize + 3] == 's'
                        && chars[self.counter as usize + 4] == 'e'
                    {
                        v.push(Token {
                            typ: Type::FALSE,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 5,
                                line_end: self.line,
                            },
                        });
                        self.counter += 4;
                        self.index += 4;
                    } else if chars[self.counter as usize + 1] == 'o'
                        && chars[self.counter as usize + 2] == 'r'
                    {
                        v.push(Token {
                            typ: Type::FOR,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 3,
                                line_end: self.line,
                            },
                        });
                        self.counter += 2;
                        self.index += 2;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                'i' => {
                    if chars[self.counter as usize + 1] == 'f' {
                        v.push(Token {
                            typ: Type::IF,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 2,
                                line_end: self.line,
                            },
                        });
                        self.counter += 1;
                        self.index += 1;
                    } else if chars[self.counter as usize + 1] == 'n' {
                        v.push(Token {
                            typ: Type::IN,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 2,
                                line_end: self.line,
                            },
                        });
                        self.counter += 1;
                        self.index += 1;
                    } else if chars[self.counter as usize + 1] == 'm'
                        && chars[self.counter as usize + 2] == 'p'
                        && chars[self.counter as usize + 3] == 'o'
                        && chars[self.counter as usize + 4] == 'r'
                        && chars[self.counter as usize + 5] == 't'
                        && !chars[self.counter as usize + 6].is_alphabetic()
                    {
                        v.push(Token {
                            typ: Type::IMPORT,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 6,
                                line_end: self.line,
                            },
                        });
                        self.counter += 5;
                        self.index += 5;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                'e' => {
                    if chars[self.counter as usize + 1] == 'n'
                        && chars[self.counter as usize + 2] == 'd'
                    {
                        v.push(Token {
                            typ: Type::END,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 3,
                                line_end: self.line,
                            },
                        });
                        self.counter += 2;
                        self.index += 2;
                    } else if chars[self.counter as usize + 1] == 'l'
                        && chars[self.counter as usize + 2] == 's'
                        && chars[self.counter as usize + 3] == 'e'
                    {
                        v.push(Token {
                            typ: Type::ELSE,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 4,
                                line_end: self.line,
                            },
                        });
                        self.counter += 3;
                        self.index += 3;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                'a' => {
                    if chars[self.counter as usize + 1] == 's'
                        && chars[self.counter as usize + 2] == 's'
                        && chars[self.counter as usize + 3] == 'e'
                        && chars[self.counter as usize + 4] == 'r'
                        && chars[self.counter as usize + 5] == 't'
                    {
                        v.push(Token {
                            typ: Type::ASSERT,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 6,
                                line_end: self.line,
                            },
                        });
                        self.counter += 5;
                        self.index += 5;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                'l' => {
                    if chars[self.counter as usize + 1] == 'e'
                        && chars[self.counter as usize + 2] == 't'
                    {
                        v.push(Token {
                            typ: Type::LET,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 3,
                                line_end: self.line,
                            },
                        });
                        self.counter += 2;
                        self.index += 2;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                't' => {
                    if chars[self.counter as usize + 1] == 'h'
                        && chars[self.counter as usize + 2] == 'e'
                        && chars[self.counter as usize + 3] == 'n'
                    {
                        v.push(Token {
                            typ: Type::THEN,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 4,
                                line_end: self.line,
                            },
                        });
                        self.counter += 3;
                        self.index += 3;
                    } else if chars[self.counter as usize + 1] == 'e'
                        && chars[self.counter as usize + 2] == 's'
                        && chars[self.counter as usize + 3] == 't'
                    {
                        v.push(Token {
                            typ: Type::TEST,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 4,
                                line_end: self.line,
                            },
                        });
                        self.counter += 3;
                        self.index += 3;
                    } else if chars[self.counter as usize + 1] == 'r'
                        && chars[self.counter as usize + 2] == 'u'
                        && chars[self.counter as usize + 3] == 'e'
                    {
                        v.push(Token {
                            typ: Type::TRUE,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 4,
                                line_end: self.line,
                            },
                        });
                        self.counter += 3;
                        self.index += 3;
                    } else if chars[self.counter as usize + 1] == 'y'
                        && chars[self.counter as usize + 2] == 'p'
                        && chars[self.counter as usize + 3] == 'e'
                    {
                        v.push(Token {
                            typ: Type::TYPE,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 4,
                                line_end: self.line,
                            },
                        });
                        self.counter += 3;
                        self.index += 3;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                'p' => {
                    if chars[self.counter as usize + 1] == 'a'
                        && chars[self.counter as usize + 2] == 's'
                        && chars[self.counter as usize + 3] == 's'
                    {
                        v.push(Token {
                            typ: Type::PASS,
                            pos: Position {
                                index: self.index,
                                line: self.index,
                                index_end: self.index + 4,
                                line_end: self.line,
                            },
                        });
                        self.counter += 3;
                        self.index += 3;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                'r' => {
                    if chars[self.counter as usize + 1] == 'e'
                        && chars[self.counter as usize + 2] == 't'
                        && chars[self.counter as usize + 3] == 'u'
                        && chars[self.counter as usize + 4] == 'r'
                        && chars[self.counter as usize + 5] == 'n'
                    {
                        v.push(Token {
                            typ: Type::RETURN,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 6,
                                line_end: self.line,
                            },
                        });
                        self.counter += 5;
                        self.index += 5;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                'u' => {
                    if chars[self.counter as usize + 1] == '3'
                        && chars[self.counter as usize + 2] == '2'
                    {
                        v.push(Token {
                            typ: Type::U32,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 3,
                                line_end: self.line,
                            },
                        });
                        self.counter += 2;
                        self.index += 2;
                    } else {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                }
                ',' => v.push(Token {
                    typ: Type::COMMA,
                    pos: Position {
                        index: self.index,
                        line: self.line,
                        index_end: self.index + 1,
                        line_end: self.line,
                    },
                }),
                '(' => v.push(Token {
                    typ: Type::LPAREN,
                    pos: Position {
                        index: self.index,
                        line: self.line,
                        index_end: self.index + 1,
                        line_end: self.line,
                    },
                }),
                ')' => v.push(Token {
                    typ: Type::RPAREN,
                    pos: Position {
                        index: self.index,
                        line: self.line,
                        index_end: self.index + 1,
                        line_end: self.line,
                    },
                }),
                '=' => {
                    if chars[self.counter as usize + 1] == '=' {
                        v.push(Token {
                            typ: Type::EQUALS,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 2,
                                line_end: self.line,
                            },
                        });
                        self.index += 1;
                        self.counter += 1;
                    } else {
                        v.push(Token {
                            typ: Type::ASSIGN,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        })
                    }
                }
                '!' => {
                    if chars[self.counter as usize + 1] == '=' {
                        v.push(Token {
                            typ: Type::NOT_EQUALS,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 2,
                                line_end: self.line,
                            },
                        });
                        self.index += 1;
                        self.counter += 1;
                    } else {
                        v.push(Token {
                            typ: Type::EXCLAIM,
                            pos: Position {
                                index: self.index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        });
                    }
                }
                '"' => {
                    // fixme deal with multiline
                    let tmp_index = self.index;
                    let mut s = "".to_string();
                    self.index += 1;
                    self.counter += 1;

                    while self.counter < chars.len().try_into().unwrap() {
                        let next = chars[self.counter as usize];
                        if next == '"' {
                            break;
                        };
                        s.push(next);
                        self.index += 1;
                        self.counter += 1;
                    }
                    v.push(Token {
                        typ: Type::STRING(s.into()),
                        pos: Position {
                            index: tmp_index,
                            line: self.line,
                            index_end: self.index,
                            line_end: self.line,
                        },
                    });
                }
                ':' => {
                    self.index += 1;
                    self.counter += 1;
                    if current.is_alphabetic() {
                        let tmp_index = self.index;
                        let mut identifier = "".to_string();
                        while self.counter < chars.len().try_into().unwrap() {
                            if !(chars[self.counter as usize].is_alphabetic()
                                || chars[self.counter as usize] == '_')
                            {
                                break;
                            }
                            let next = chars[self.counter as usize];
                            identifier.push(next);
                            self.index += 1;
                            self.counter += 1;
                        }
                        // identifier
                        v.push(Token {
                            typ: Type::ATOM(identifier.into()),
                            pos: Position {
                                index: tmp_index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        });
                        continue;
                    }
                }
                _ => {
                    if current.is_alphabetic() || current == '_' {
                        self.identifier(&chars, &mut v);
                        continue;
                    }
                    if current.is_numeric() {
                        let mut identifier = "".to_string();
                        let tmp_index = self.index;
                        while self.counter < chars.len().try_into().unwrap() {
                            if chars[self.counter as usize].is_whitespace()
                                || !chars[self.counter as usize].is_numeric()
                            {
                                break;
                            }
                            let next = chars[self.counter as usize];
                            identifier.push(next);
                            self.index += 1;
                            self.counter += 1;
                        }
                        // identifier
                        v.push(Token {
                            typ: Type::NUMBER(identifier.into()),
                            pos: Position {
                                index: tmp_index,
                                line: self.line,
                                index_end: self.index + 1,
                                line_end: self.line,
                            },
                        });
                        continue;
                    }
                }
            }

            self.index += 1;
            self.counter += 1;
        }

        return v;
    }

    fn identifier(&mut self, chars: &Vec<char>, v: &mut Vec<Token>) {
        let tmp_index = self.index;
        let mut identifier = "".to_string();
        while self.counter < chars.len().try_into().unwrap() {
            if !(chars[self.counter as usize].is_alphabetic()
                || chars[self.counter as usize] == '_'
                || chars[self.counter as usize].is_numeric())
            {
                break;
            }
            let next = chars[self.counter as usize];
            identifier.push(next);
            self.index += 1;
            self.counter += 1;
        }
        // identifier
        v.push(Token {
            typ: Type::IDENTIFIER(identifier.into()),
            pos: Position {
                index: tmp_index,
                line: self.line,
                index_end: self.index + 1,
                line_end: self.line,
            },
        });
    }
}
