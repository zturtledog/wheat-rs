// the impl for wheat as a dylib

pub mod wheat {
    use std::fs;

    use regex::Regex;

    pub struct Tofi {
        pub ast: Vec<Node>,
        pub src: String,
    }

    #[derive(Debug, Clone)]
    pub struct Node {
        pub line: usize,
        pub col : usize,
        pub val_str : Option<Token>,
        pub val_vec : Option<Vec<Node>>,
        pub is_block : bool,
        depth: usize
    }
    impl Node {
        pub fn new_str(line: usize, col: usize, sr: Token) -> Self {
            Node {
                line,
                col,
                val_str: Some(sr),
                val_vec: None,
                is_block: false,
                depth:0
            }
        }

        pub fn new_vec(line: usize, col: usize, sr: Vec<Node>, depth: usize) -> Self {
            Node {
                line,
                col,
                val_str: None,
                val_vec: Some(sr),
                is_block: true,
                depth
            }
        }

        pub fn is_block(&self) -> bool {
            self.is_block
        }
    }

    #[derive(Debug, Clone)]
    pub enum Tktype {
        SPECIAL(String),
        STRING,
        TYPE,
        NUMBER,
        OPERATOR,
        INCREMENTOR,
        DECREMENTOR,
        CALL,
        POINTER,
        WORD,
        WHITESPACE,
        NEWLINE,
        UNSEAR,
    }
    impl Tktype {
        fn color(&self, inp: String) -> String {
            return match self {
                Tktype::STRING => format!("\x1b[92m{}\x1b[0m", inp),
                Tktype::NUMBER => format!("\x1b[93m{}\x1b[0m", inp),
                Tktype::CALL => format!("\x1b[95m{}\x1b[0m", inp),
                Tktype::UNSEAR => format!("\x1b[90m{}\x1b[0m", inp),
                Tktype::OPERATOR => format!("\x1b[91m{}\x1b[0m", inp),
                Tktype::INCREMENTOR => format!("\x1b[91m{}\x1b[0m", inp),
                Tktype::DECREMENTOR => format!("\x1b[91m{}\x1b[0m", inp),
                Tktype::TYPE => format!("\x1b[94m{}\x1b[0m", inp),
                Tktype::POINTER => format!("\x1b[93m{}\x1b[0m", inp),
                _ => inp,
            };
        }
    }

    #[derive(Debug, Clone)]
    pub struct Token {
        offset: usize,
        tpe: Tktype,
        value: String,
    }
    impl Token {
        fn new(off: usize, tpe: Tktype, value: &String) -> Self {
            return Token {
                offset: off,
                tpe: tpe,
                value: value.to_string(),
            };
        }

        fn dup(&self) -> Token {
            return Token {
                offset: self.offset,
                tpe: self.tpe.clone(),
                value: self.value.clone(),
            };
        }
    }

    struct Pattern {
        name: Tktype,
        exp: Regex,
    }
    fn npat<'a>(name: Tktype, exp: &'a str) -> Pattern {
        return Pattern {
            name: name,
            exp: Regex::new(exp).unwrap(), //.expect("regex failed to compile")
        };
    }

    pub fn load(path: String) -> Tofi {
        let mat: Vec<Pattern> = vec![
            // npat("comment", "#.*?(#|$)"),//[#](?:\\\\[#\\\\]|[^\\n#\\\\])*[#|\\n]
            npat(Tktype::STRING, "['](?:\\\\['\\\\]|[^\\n'\\\\])*[']"),
            npat(Tktype::STRING, "[\"](?:\\\\[\"\\\\]|[^\\n\"\\\\])*[\"]"),
            npat(Tktype::NEWLINE, "\r?\n"),
            npat(Tktype::TYPE, ":[a-zA-Z_]+:"),
            npat(Tktype::NUMBER, "-?(?:0|[1-9][0-9]*)\\.?(?:0|[1-9][0-9]*)?"),
            npat(Tktype::INCREMENTOR, "\\+\\+"),
            npat(Tktype::DECREMENTOR, "--"),
            npat(Tktype::OPERATOR, "\\+"),
            npat(Tktype::OPERATOR, "-"),
            npat(Tktype::OPERATOR, "\\*"),
            npat(Tktype::OPERATOR, "/"),
            npat(Tktype::OPERATOR, "\\|\\|"),
            npat(Tktype::OPERATOR, "&&"),
            npat(Tktype::OPERATOR, "%"),
            npat(Tktype::OPERATOR, "\\^"),
            npat(Tktype::OPERATOR, "<"),
            npat(Tktype::OPERATOR, ">"),
            npat(Tktype::OPERATOR, "<<"),
            npat(Tktype::OPERATOR, ">>"),
            npat(Tktype::OPERATOR, "=="),
            npat(Tktype::OPERATOR, "!="),
            npat(Tktype::SPECIAL(String::from("comma")), ","),
            npat(Tktype::SPECIAL(String::from("bl-open")), "\\{"),
            npat(Tktype::SPECIAL(String::from("bl-close")), "\\}"),
            npat(Tktype::SPECIAL(String::from("pr-open")), "\\("),
            npat(Tktype::SPECIAL(String::from("pr-close")), "\\)"),
            npat(Tktype::SPECIAL(String::from("array")), "\\[((?:0|[1-9][0-9]*)\\.?(?:0|[1-9][0-9]*)?)?\\]"),
            npat(Tktype::CALL, "\\.[a-zA-Z_]+"),
            npat(Tktype::WHITESPACE, "(\\s|\\t)+"),
            npat(Tktype::POINTER, "~[a-zA-Z_]+"),
            npat(Tktype::WORD, "[a-zA-Z_]+"),
        ];

        let file = patchcomments(fs::read_to_string(&path).unwrap());
        let mut tkns = vec![Token::new(0, Tktype::UNSEAR, &file)];
        mat.iter().for_each(|pat| {
            let mut temp: Vec<Token> = vec![];
            tkns.iter().for_each(|mat| match mat.tpe {
                Tktype::UNSEAR => {
                    matchpattern(&mat.value, pat, mat.offset)
                        .iter()
                        .for_each(|matm| temp.push(matm.dup()));
                }
                _ => {
                    temp.push(mat.dup());
                }
            });
            tkns = temp;
        });

        Tofi {
            ast: gen_ast(&tkns),
            src: path,
        }
    }

    // this is almost certainly bad practise
    fn patchcomments(data: String) -> String {
        let mut record = String::from("");

        let mut instring = false;
        let mut incomment = false;

        let code = data.as_bytes();

        let mut p1 = ' ';
        let mut p2 = ' ';
        for i in 0..code.len() as isize {
            if i - 1 >= 0 {
                p1 = code[(i - 1) as usize] as char
            }
            if i - 2 >= 0 {
                p2 = code[(i - 2) as usize] as char
            }
            let curr = code[(i as usize)] as char;

            if (curr == '\'' || curr == '"') && !(p1 == '\\' && p2 != '\\') && !incomment {
                instring = !instring;
            } else if curr == '#' && !instring {
                incomment = !incomment;
                record.push(' ');
                continue;
            } else if (curr == '\n' || curr == '\r') && incomment {
                incomment = false;
            }

            if !incomment {
                record.push(curr);
            } else {
                record.push(' ');
            }
        }

        if instring {
            error("during pre-parse: no position info","the parser found an unclosed string");
            // std::process::exit(0);
        }

        record
    }

    fn matchpattern(data: &String, pat: &Pattern, off: usize) -> Vec<Token> {
        let mut temp: Vec<Token> = vec![];
        let mut end = 0;
        pat.exp.find_iter(data).for_each(|mat| {
            let start = &data[end..mat.start()];
            let middle = mat.as_str();

            if start != "" {
                temp.push(Token::new(end + off, Tktype::UNSEAR, &start.to_string()))
            }
            temp.push(Token::new(
                mat.start() + off,
                pat.name.clone(),
                &middle.to_string(),
            ));

            end = mat.end();
        });
        let last = &data[end..data.len()];
        if last != "" {
            temp.push(Token::new(end + off, Tktype::UNSEAR, &last.to_string()))
        }

        temp
    }

    fn gen_ast(tkns: &Vec<Token>) -> Vec<Node> {
        //loop
        //mark depth and max depth
        //mar line and col
        //loop from max depth to 0
        //  compqt

        let mut temp : Vec<Node> = vec![];
        let mut line: usize = 0;
        let mut coltrack: usize = 0;
        let mut colrec: usize = 0;
        let mut depth = 0;
        tkns.iter().for_each(|tks| {
            temp.push(Node::new_str(line, tks.offset, tks.dup()));

            match &tks.tpe {
                Tktype::NEWLINE => {
                    coltrack += colrec;
                    line += 1
                },
                Tktype::SPECIAL(id) => {
                    let indx = &temp.len()-1;
                    if id == &String::from("bl-open") || id == &String::from("pr-open") {
                        depth += 1;
                        temp[indx].depth = depth
                    }
                    
                    if id == &String::from("bl-close") || id == &String::from("pr-close") {
                        temp[indx].depth = depth;
                        depth -= 1
                    }

                    temp[indx].is_block = true;
                },
                _=>{}
            } 
            colrec += tks.value.len();

        });

        temp.iter().for_each(|f| {
            let tk = f.val_str.as_ref().expect("pain");
            // if !f.is_block() {
            print!("\x1b[90m({}-{}{})\x1b[91m{}\x1b[0m",f.line,f.col, if f.is_block() {
                format!("[{}]",f.depth)
            } else {
                "".to_string()
            } ,tk.value.clone())//tk.tpe.color(tk.value.clone())
            // }
        });

        temp
    }

    fn error(at: &str, message: &str) {
        println!("\x1b[91merror\n\x1b[0m  where: \x1b[96m{}\x1b[0m\n  message: \x1b[92m{}\x1b[0m\n",at,message)
    }

    fn dumptokens(tkns: &Vec<Token>, indx: bool) {
        print!("\n");
        tkns.iter().for_each(|token| {
            if indx {
                print!(
                    "\x1b[91m{}\x1b[90m({}-{})\x1b[0m",
                    token.value.clone(),
                    token.offset,
                    token.value.len()
                );
            } else {
                print!("{}", token.tpe.color(token.value.clone()));
            }
        });
        print!("\n")
    }
}

/*
function tokenlines(sta, r) {
    // print!("\x1b[93m{}\x1b[0m({})",start,end)
    // print!("\x1b[91m{}\x1b[0m({})",middle,mat.start());
    // println!("\x1b[92m{}\x1b[0m",last)
  let boffer = [{ type: "unsear", value: sta, constructor:{name:"unlexed"} }];
  for (let i = 0; i < r.mat.length; i++) {
    //if match then
    //  recurse
    //  mixlists
    //  break

    if (sta.search(r.mat[i][1]) > -1) {
      let result = sta.search(r.mat[i][1]);
      let front = tokenlines(sta.substring(0, result), r);
      let token = {
        type: r.mat[i][0],
        value: (r.callback && r.callback[r.mat[i][0]]) ?
          r.callback[r.mat[i][0]](
            sta.substring(result, sta.length)
              .match(r.mat[i][1])[0])
          : sta.substring(result, sta.length)
            .match(r.mat[i][1])[0],
        constructor: {
          name:r.mat[i][0]
        },
      };
      let back = tokenlines(
        sta.substring(result, sta.length)
          .replace(r.mat[i][1], ""), r);

      front.push(token);
      boffer = mixlist(front, back);

      let bofferaswell = [];
      for (let j = 0; j < boffer.length; j++) {
        if (boffer[j].value != "") {
          bofferaswell.push(boffer[j]);
        }
      }
      boffer = bofferaswell;

      return boffer;
    }
  }
  return boffer;
}

 */

/*
impl Tkval {
       fn get_str(&self) -> Option<String> {
           return match self {
               Tkval::OF(x) => Some(x.to_string()),
               Tkval::ANY(_s) => None,
           }
       }
       fn get_vec(&self) -> Option<Vec<Token>> {
           return match self {
               Tkval::OF(_s) => None,
               Tkval::ANY(x) => Some(x.to_vec()),
           }
       }
   }
// */
