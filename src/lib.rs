// the impl for wheat as a dylib

pub mod wheat {
    use std::{fs, vec};

    use regex::Regex;

    pub struct Tofi {
        tokens: Vec<Token>,
        src: String,
    }

    #[derive(Debug)]
    struct Token {
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

    #[derive(Debug)]
    struct Pattern {
        name: Tktype,
        exp: Regex,
    }

    pub fn load(path: String) -> Option<Tofi> {
        let mat = vec![
            // npat("comment", "#.*?(#|$)"),//[#](?:\\\\[#\\\\]|[^\\n#\\\\])*[#|\\n]
            npat(Tktype::STRING, "['](?:\\\\['\\\\]|[^\\n'\\\\])*[']"),
            npat(Tktype::STRING, "[\"](?:\\\\[\"\\\\]|[^\\n\"\\\\])*[\"]"),
            npat(Tktype::TYPE, ":[a-zA-Z_]+:"),
            npat(Tktype::NUMBER, "-?(?:0|[1-9][0-9]*)\\.?(?:0|[1-9][0-9]*)?"),
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
            npat(Tktype::SPECIAL(String::from("ar-open")), "\\["),
            npat(Tktype::SPECIAL(String::from("ar-close")), "\\]"),
            npat(Tktype::INCREMENTOR, "\\+\\+"),
            npat(Tktype::DECREMENTOR, "--"),
            npat(Tktype::CALL, "\\.[a-zA-Z_]+"),
            npat(Tktype::WHITESPACE, "(\\s|\\t)+"),
            npat(Tktype::POINTER, "~[a-zA-Z_]+"),
            npat(Tktype::WORD, "[a-zA-Z_]+"),
            npat(Tktype::NEWLINE, "\r?\n"),
        ];

        let file = fs::read_to_string(path).unwrap();
        let mut tkns = vec![Token::new(0, Tktype::UNSEAR, &file)];
        mat.iter().for_each(|pat| {
            let mut temp: Vec<Token> = vec![];
            tkns.iter().for_each(|mat| {
                match mat.tpe {
                    Tktype::UNSEAR => {
                        matchpattern(&mat.value, pat, mat.offset).iter().for_each(|matm| {
                            temp.push(matm.dup())
                        });
                    },
                    _ => {
                        temp.push(mat.dup());
                    }
                }
            });
            tkns = temp;
        });

        dumptokens(tkns,!true);

        return None;
    }

    fn npat<'a>(name: Tktype, exp: &'a str) -> Pattern {
        return Pattern {
            name: name,
            exp: Regex::new(exp).unwrap(), //.expect("regex failed to compile")
        };
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

    #[derive(Debug, Clone)]
    enum Tktype {
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
                Tktype::TYPE => format!("\x1b[94m{}\x1b[0m", inp),
                _ => inp,
            };
        }
    }

    fn dumptokens(tkns: Vec<Token>, indx: bool) {
        print!("\n");
        tkns.iter().for_each(|token| {
            if indx {
                print!("{}\x1b[90m({}-{})\x1b[0m", token.tpe.color(token.value.clone()), token.offset,token.value.len());
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
