
#[derive(Debug, Clone)]
pub enum Tree {
    Var(usize),
    Abs(usize, Box<Tree>),
    App(Box<Tree>, Box<Tree>)
}

impl Tree {
    #[allow(dead_code)]
    pub fn to_indexed_string(&self) -> String {
        let mut result = String::new();
        match self {
            Tree::Var(index) => result.push_str(&index.to_string()),
            Tree::Abs(_, body) => {
                result.push('(');
                result.push_str(&"λ");
                let mut temp = body.to_indexed_string();
                result.extend(temp.drain(..));
                result.push(')');
            },
            Tree::App(left, right) => {
                result.push('(');
                let mut temp = left.to_indexed_string();
                result.extend(temp.drain(..));

                result.push(' ');
                let mut temp = right.to_indexed_string();
                result.extend(temp.drain(..));
                result.push(')');
            }
        }
        result
    }

    pub fn convert(&self) -> Option<u64> {
        let mut level = 0;
        let mut rec = self;
        loop {
            rec = match rec {
                Tree::Var(_) => {
                    return Some(level - 2);
                },
                Tree::Abs(_, body) => {
                    if level < 2 {
                        level += 1;
                        &*body
                    } else {
                        return None;
                    }
                },
                Tree::App(left, right) => {
                    if let Tree::Var(_) = **left {
                        level += 1;
                        &*right
                    } else {
                        return None;
                    }
                }
            };
        }
    }
}
